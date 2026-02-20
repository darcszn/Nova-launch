import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { StellarService } from '../stellar';

vi.mock('@stellar/stellar-sdk', () => ({
  Address: {
    fromString: function(addr: string) {
      if (!addr.startsWith('C') || addr.length !== 56) throw new Error('Invalid');
      return addr;
    },
  },
  Contract: function() { return {}; },
  TransactionBuilder: function() { return {}; },
  BASE_FEE: '100',
  Networks: {
    TESTNET: 'Test SDF Network ; September 2015',
    PUBLIC: 'Public Global Stellar Network ; September 2015',
  },
  SorobanRpc: {
    Server: function() {
      return {
        getAccount: vi.fn(),
        simulateTransaction: vi.fn(),
      };
    },
    Api: {
      isSimulationSuccess: vi.fn(),
    },
  },
}));

describe('StellarService', () => {
  let service: StellarService;

  beforeEach(() => {
    vi.clearAllMocks();
    global.fetch = vi.fn();
    service = new StellarService('testnet');
  });

  describe('getTokenInfo', () => {
    it('throws error for invalid address', async () => {
      await expect(service.getTokenInfo('invalid')).rejects.toThrow('Invalid token address');
    });

    it('returns token info without metadata', async () => {
      const mockAddress = 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC';

      (global.fetch as Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          _embedded: {
            records: [{
              hash: 'txhash123',
              source_account: 'GABC...',
              created_at: '2026-01-01T00:00:00Z',
            }],
          },
        }),
      });

      const info = await service.getTokenInfo(mockAddress);

      expect(info.address).toBe(mockAddress);
      expect(info.creator).toBe('GABC...');
      expect(info.transactionHash).toBe('txhash123');
      expect(info.metadataUri).toBeUndefined();
    });
  });

  describe('monitorTransaction', () => {
    it('returns success when transaction is found', async () => {
      const hash = 'abc123';
      
      (global.fetch as Mock).mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          successful: true,
          created_at: '2026-01-01T00:00:00Z',
          fee_charged: '100',
        }),
      });

      const result = await service.monitorTransaction(hash);

      expect(result.status).toBe('success');
      expect(result.hash).toBe(hash);
      expect(result.fee).toBe('100');
    });

    it('polls until transaction is found', async () => {
      const hash = 'abc123';
      const onProgress = vi.fn();

      (global.fetch as Mock)
        .mockResolvedValueOnce({ status: 404 })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            successful: true,
            created_at: '2026-01-01T00:00:00Z',
            fee_charged: '100',
          }),
        });

      const result = await service.monitorTransaction(hash, onProgress);

      expect(result.status).toBe('success');
      expect(onProgress).toHaveBeenCalled();
    });

    it('throws error on timeout', async () => {
      const hash = 'abc123';
      
      (global.fetch as Mock).mockResolvedValue({ status: 404 });

      await expect(
        service.monitorTransaction(hash)
      ).rejects.toThrow('Transaction monitoring timeout');
    }, 70000);
  });
});
