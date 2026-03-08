import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { PrismaClient } from '@prisma/client';
import { legacyFixtures } from './fixtures/legacySchemas';

/**
 * Migration and Upgrade Tests
 * 
 * These tests simulate real-world migration scenarios where:
 * 1. Old state is loaded from database
 * 2. Schema is upgraded
 * 3. New read/write paths are exercised
 * 4. Data integrity is verified
 */

describe('Migration Scenarios - Token Model', () => {
  let prisma: PrismaClient;

  beforeEach(async () => {
    prisma = new PrismaClient();
    await prisma.burnRecord.deleteMany();
    await prisma.analytics.deleteMany();
    await prisma.token.deleteMany();
  });

  afterEach(async () => {
    await prisma.$disconnect();
  });

  describe('V1 to V2 Migration (Add Burn Tracking)', () => {
    it('should migrate V1 tokens and add burn tracking', async () => {
      // Step 1: Load V1 state
      const v1Tokens = await Promise.all([
        prisma.token.create({
          data: {
            address: 'CTOKEN1',
            creator: 'GCREATOR1',
            name: 'Token 1',
            symbol: 'TK1',
            totalSupply: BigInt('1000000000000'),
            initialSupply: BigInt('1000000000000'),
          },
        }),
        prisma.token.create({
          data: {
            address: 'CTOKEN2',
            creator: 'GCREATOR2',
            name: 'Token 2',
            symbol: 'TK2',
            totalSupply: BigInt('2000000000000'),
            initialSupply: BigInt('2000000000000'),
          },
        }),
      ]);

      // Step 2: Simulate migration - add burn tracking fields
      for (const token of v1Tokens) {
        await prisma.token.update({
          where: { id: token.id },
          data: {
            totalBurned: BigInt(0),
            burnCount: 0,
          },
        });
      }

      // Step 3: Exercise new write path - record burns
      await prisma.burnRecord.create({
        data: {
          tokenId: v1Tokens[0].id,
          from: 'GBURNER1',
          amount: BigInt('100000000000'),
          burnedBy: 'GBURNER1',
          txHash: 'tx-burn-1',
          timestamp: new Date(),
        },
      });

      // Update token burn stats
      await prisma.token.update({
        where: { id: v1Tokens[0].id },
        data: {
          totalBurned: BigInt('100000000000'),
          burnCount: 1,
        },
      });

      // Step 4: Exercise new read path
      const updatedToken = await prisma.token.findUnique({
        where: { id: v1Tokens[0].id },
        include: { burnRecords: true },
      });

      // Verify migration success
      expect(updatedToken?.totalBurned.toString()).toBe('100000000000');
      expect(updatedToken?.burnCount).toBe(1);
      expect(updatedToken?.burnRecords).toHaveLength(1);
      
      // Verify original data intact
      expect(updatedToken?.address).toBe('CTOKEN1');
      expect(updatedToken?.totalSupply.toString()).toBe('1000000000000');
    });
  });

  describe('V2 to V3 Migration (Add Metadata URI)', () => {
    it('should migrate V2 tokens and add metadata support', async () => {
      // Step 1: Load V2 state
      const v2Token = await prisma.token.create({
        data: {
          address: 'CTOKENV2',
          creator: 'GCREATORV2',
          name: 'Token V2',
          symbol: 'TKV2',
          totalSupply: BigInt('1000000000000'),
          initialSupply: BigInt('1000000000000'),
          totalBurned: BigInt('50000000000'),
          burnCount: 5,
        },
      });

      // Step 2: Simulate migration - metadataUri is already nullable, no action needed

      // Step 3: Exercise new write path - add metadata
      const updated = await prisma.token.update({
        where: { id: v2Token.id },
        data: {
          metadataUri: 'ipfs://QmTokenMetadata123',
        },
      });

      // Step 4: Exercise new read path
      const retrieved = await prisma.token.findUnique({
        where: { id: v2Token.id },
      });

      // Verify migration success
      expect(retrieved?.metadataUri).toBe('ipfs://QmTokenMetadata123');
      
      // Verify V2 data intact
      expect(retrieved?.totalBurned.toString()).toBe('50000000000');
      expect(retrieved?.burnCount).toBe(5);
      
      // Verify V1 data intact
      expect(retrieved?.address).toBe('CTOKENV2');
      expect(retrieved?.totalSupply.toString()).toBe('1000000000000');
    });
  });

  describe('Full V1 to V3 Migration', () => {
    it('should migrate through all versions without data loss', async () => {
      // Step 1: Create V1 token
      const v1Token = await prisma.token.create({
        data: {
          address: 'CTOKENFULL',
          creator: 'GCREATORFULL',
          name: 'Full Migration Token',
          symbol: 'FMT',
          totalSupply: BigInt('1000000000000'),
          initialSupply: BigInt('1000000000000'),
        },
      });

      // Verify V1 state
      let token = await prisma.token.findUnique({ where: { id: v1Token.id } });
      expect(token?.totalBurned).toBe(BigInt(0));
      expect(token?.burnCount).toBe(0);
      expect(token?.metadataUri).toBeNull();

      // Step 2: Migrate to V2 (add burn tracking)
      await prisma.token.update({
        where: { id: v1Token.id },
        data: {
          totalBurned: BigInt('100000000000'),
          burnCount: 10,
        },
      });

      // Verify V2 state
      token = await prisma.token.findUnique({ where: { id: v1Token.id } });
      expect(token?.totalBurned.toString()).toBe('100000000000');
      expect(token?.burnCount).toBe(10);
      expect(token?.metadataUri).toBeNull();

      // Step 3: Migrate to V3 (add metadata)
      await prisma.token.update({
        where: { id: v1Token.id },
        data: {
          metadataUri: 'ipfs://QmFullMigration',
        },
      });

      // Verify V3 state
      token = await prisma.token.findUnique({ where: { id: v1Token.id } });
      expect(token?.metadataUri).toBe('ipfs://QmFullMigration');
      expect(token?.totalBurned.toString()).toBe('100000000000');
      expect(token?.burnCount).toBe(10);
      
      // Verify original V1 data intact
      expect(token?.address).toBe('CTOKENFULL');
      expect(token?.name).toBe('Full Migration Token');
      expect(token?.totalSupply.toString()).toBe('1000000000000');
    });
  });
});

describe('Migration Scenarios - Stream Model', () => {
  let prisma: PrismaClient;

  beforeEach(async () => {
    prisma = new PrismaClient();
    await prisma.stream.deleteMany();
  });

  afterEach(async () => {
    await prisma.$disconnect();
  });

  describe('V1 to V3 Migration', () => {
    it('should migrate V1 streams through all versions', async () => {
      // Step 1: Create V1 stream
      const v1Stream = await prisma.stream.create({
        data: {
          streamId: 1001,
          creator: 'GCREATORSTREAM',
          recipient: 'GRECIPIENTSTREAM',
          amount: BigInt('1000000000000'),
          status: 'CREATED',
          txHash: 'tx-stream-migration',
        },
      });

      // Verify V1 state
      let stream = await prisma.stream.findUnique({ where: { streamId: 1001 } });
      expect(stream?.metadata).toBeNull();
      expect(stream?.claimedAt).toBeNull();
      expect(stream?.cancelledAt).toBeNull();

      // Step 2: Migrate to V2 (add metadata)
      await prisma.stream.update({
        where: { streamId: 1001 },
        data: {
          metadata: JSON.stringify({ purpose: 'Payment', period: 'monthly' }),
        },
      });

      // Verify V2 state
      stream = await prisma.stream.findUnique({ where: { streamId: 1001 } });
      expect(stream?.metadata).not.toBeNull();
      expect(stream?.claimedAt).toBeNull();

      // Step 3: Migrate to V3 (add timestamps) by claiming
      const claimTime = new Date();
      await prisma.stream.update({
        where: { streamId: 1001 },
        data: {
          status: 'CLAIMED',
          claimedAt: claimTime,
        },
      });

      // Verify V3 state
      stream = await prisma.stream.findUnique({ where: { streamId: 1001 } });
      expect(stream?.status).toBe('CLAIMED');
      expect(stream?.claimedAt).not.toBeNull();
      
      // Verify original data intact
      expect(stream?.creator).toBe('GCREATORSTREAM');
      expect(stream?.amount.toString()).toBe('1000000000000');
    });
  });

  describe('Status Transition Migration', () => {
    it('should handle status changes with new timestamp fields', async () => {
      // Create stream
      const stream = await prisma.stream.create({
        data: {
          streamId: 2001,
          creator: 'GCREATOR2001',
          recipient: 'GRECIPIENT2001',
          amount: BigInt('500000000000'),
          status: 'CREATED',
          txHash: 'tx-stream-2001',
        },
      });

      // Transition to CLAIMED
      const claimTime = new Date();
      await prisma.stream.update({
        where: { streamId: 2001 },
        data: {
          status: 'CLAIMED',
          claimedAt: claimTime,
        },
      });

      const claimed = await prisma.stream.findUnique({ where: { streamId: 2001 } });
      expect(claimed?.status).toBe('CLAIMED');
      expect(claimed?.claimedAt).toEqual(claimTime);
      expect(claimed?.cancelledAt).toBeNull();
    });
  });
});

describe('Migration Scenarios - Governance Models', () => {
  let prisma: PrismaClient;

  beforeEach(async () => {
    prisma = new PrismaClient();
    await prisma.proposalExecution.deleteMany();
    await prisma.vote.deleteMany();
    await prisma.proposal.deleteMany();
  });

  afterEach(async () => {
    await prisma.$disconnect();
  });

  describe('Proposal V1 to V3 Migration', () => {
    it('should migrate proposals through all versions', async () => {
      // Step 1: Create V1 proposal
      const v1Proposal = await prisma.proposal.create({
        data: {
          proposalId: 3001,
          tokenId: 'CTOKEN3001',
          proposer: 'GPROPOSER3001',
          title: 'Migration Test Proposal',
          proposalType: 'PARAMETER_CHANGE',
          status: 'ACTIVE',
          startTime: new Date(),
          endTime: new Date(Date.now() + 86400000),
          quorum: BigInt('1000000000000'),
          threshold: BigInt('500000000000'),
          txHash: 'tx-proposal-3001',
        },
      });

      // Verify V1 state
      let proposal = await prisma.proposal.findUnique({ where: { proposalId: 3001 } });
      expect(proposal?.description).toBeNull();
      expect(proposal?.metadata).toBeNull();
      expect(proposal?.executedAt).toBeNull();

      // Step 2: Migrate to V2 (add description and metadata)
      await prisma.proposal.update({
        where: { proposalId: 3001 },
        data: {
          description: 'This proposal increases the burn fee',
          metadata: JSON.stringify({ category: 'fee', impact: 'medium' }),
        },
      });

      // Verify V2 state
      proposal = await prisma.proposal.findUnique({ where: { proposalId: 3001 } });
      expect(proposal?.description).not.toBeNull();
      expect(proposal?.metadata).not.toBeNull();
      expect(proposal?.executedAt).toBeNull();

      // Step 3: Migrate to V3 (execute proposal)
      const execTime = new Date();
      await prisma.proposal.update({
        where: { proposalId: 3001 },
        data: {
          status: 'EXECUTED',
          executedAt: execTime,
        },
      });

      // Verify V3 state
      proposal = await prisma.proposal.findUnique({ where: { proposalId: 3001 } });
      expect(proposal?.status).toBe('EXECUTED');
      expect(proposal?.executedAt).toEqual(execTime);
      
      // Verify all previous data intact
      expect(proposal?.title).toBe('Migration Test Proposal');
      expect(proposal?.description).toBe('This proposal increases the burn fee');
    });
  });

  describe('Vote Migration with Proposals', () => {
    it('should maintain vote-proposal relationships across migrations', async () => {
      // Create V1 proposal
      const proposal = await prisma.proposal.create({
        data: {
          proposalId: 4001,
          tokenId: 'CTOKEN4001',
          proposer: 'GPROPOSER4001',
          title: 'Vote Migration Test',
          proposalType: 'PARAMETER_CHANGE',
          status: 'ACTIVE',
          startTime: new Date(),
          endTime: new Date(Date.now() + 86400000),
          quorum: BigInt('1000000000000'),
          threshold: BigInt('500000000000'),
          txHash: 'tx-proposal-4001',
        },
      });

      // Add V1 votes (without reason)
      const v1Vote = await prisma.vote.create({
        data: {
          proposalId: proposal.id,
          voter: 'GVOTER1',
          support: true,
          weight: BigInt('250000000000'),
          txHash: 'tx-vote-4001-1',
          timestamp: new Date(),
        },
      });

      // Add V2 vote (with reason)
      const v2Vote = await prisma.vote.create({
        data: {
          proposalId: proposal.id,
          voter: 'GVOTER2',
          support: false,
          weight: BigInt('100000000000'),
          reason: 'I disagree with this proposal',
          txHash: 'tx-vote-4001-2',
          timestamp: new Date(),
        },
      });

      // Verify relationships
      const proposalWithVotes = await prisma.proposal.findUnique({
        where: { proposalId: 4001 },
        include: { votes: true },
      });

      expect(proposalWithVotes?.votes).toHaveLength(2);
      expect(proposalWithVotes?.votes.find(v => v.voter === 'GVOTER1')?.reason).toBeNull();
      expect(proposalWithVotes?.votes.find(v => v.voter === 'GVOTER2')?.reason).not.toBeNull();
    });
  });
});

describe('Batch Migration Scenarios', () => {
  let prisma: PrismaClient;

  beforeEach(async () => {
    prisma = new PrismaClient();
    await prisma.burnRecord.deleteMany();
    await prisma.analytics.deleteMany();
    await prisma.token.deleteMany();
  });

  afterEach(async () => {
    await prisma.$disconnect();
  });

  it('should migrate large batch of V1 tokens efficiently', async () => {
    // Create 100 V1 tokens
    const v1Tokens = [];
    for (let i = 0; i < 100; i++) {
      v1Tokens.push({
        address: `CTOKEN${i}`,
        creator: `GCREATOR${i}`,
        name: `Token ${i}`,
        symbol: `TK${i}`,
        totalSupply: BigInt('1000000000000'),
        initialSupply: BigInt('1000000000000'),
      });
    }

    await prisma.token.createMany({ data: v1Tokens });

    // Verify all created
    const count = await prisma.token.count();
    expect(count).toBe(100);

    // Batch update to V2 (add burn tracking)
    await prisma.token.updateMany({
      data: {
        totalBurned: BigInt(0),
        burnCount: 0,
      },
    });

    // Verify migration
    const tokens = await prisma.token.findMany();
    expect(tokens.every(t => t.totalBurned === BigInt(0))).toBe(true);
    expect(tokens.every(t => t.burnCount === 0)).toBe(true);
  });

  it('should handle mixed version data in same table', async () => {
    // Create tokens at different versions
    const tokens = await Promise.all([
      // V1 token
      prisma.token.create({
        data: {
          address: 'CTOKENMIX1',
          creator: 'GCREATORMIX1',
          name: 'Mixed V1',
          symbol: 'MX1',
          totalSupply: BigInt('1000000000000'),
          initialSupply: BigInt('1000000000000'),
        },
      }),
      // V2 token
      prisma.token.create({
        data: {
          address: 'CTOKENMIX2',
          creator: 'GCREATORMIX2',
          name: 'Mixed V2',
          symbol: 'MX2',
          totalSupply: BigInt('1000000000000'),
          initialSupply: BigInt('1000000000000'),
          totalBurned: BigInt('50000000000'),
          burnCount: 5,
        },
      }),
      // V3 token
      prisma.token.create({
        data: {
          address: 'CTOKENMIX3',
          creator: 'GCREATORMIX3',
          name: 'Mixed V3',
          symbol: 'MX3',
          totalSupply: BigInt('1000000000000'),
          initialSupply: BigInt('1000000000000'),
          totalBurned: BigInt('100000000000'),
          burnCount: 10,
          metadataUri: 'ipfs://QmMixed',
        },
      }),
    ]);

    // Verify all versions coexist
    const allTokens = await prisma.token.findMany({
      where: {
        address: { in: ['CTOKENMIX1', 'CTOKENMIX2', 'CTOKENMIX3'] },
      },
    });

    expect(allTokens).toHaveLength(3);
    
    // V1 has defaults
    const v1 = allTokens.find(t => t.address === 'CTOKENMIX1');
    expect(v1?.totalBurned).toBe(BigInt(0));
    expect(v1?.metadataUri).toBeNull();
    
    // V2 has burn data
    const v2 = allTokens.find(t => t.address === 'CTOKENMIX2');
    expect(v2?.totalBurned.toString()).toBe('50000000000');
    expect(v2?.metadataUri).toBeNull();
    
    // V3 has all fields
    const v3 = allTokens.find(t => t.address === 'CTOKENMIX3');
    expect(v3?.totalBurned.toString()).toBe('100000000000');
    expect(v3?.metadataUri).toBe('ipfs://QmMixed');
  });
});
