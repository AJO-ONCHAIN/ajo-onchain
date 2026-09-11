/**
 * Environment configuration for the indexer.
 *
 * Note what is absent: there is no secret key, no signing seed, no mnemonic.
 * The Supabase key here is a service role key for writing cache rows, and the
 * WhatsApp token sends messages. Neither can move funds. If you find yourself
 * adding a Stellar secret to this interface, read the header comment in
 * `src/index.ts` first.
 */
export interface IndexerConfig {
  /** Soroban RPC endpoint. */
  rpcUrl: string;
  /** Network passphrase, e.g. "Test SDF Network ; September 2015". */
  networkPassphrase: string;
  /** The factory whose `deployed` events tell us which vaults to watch. */
  factoryContractId: string;
  supabaseUrl: string;
  supabaseServiceKey: string;
  pollIntervalMs: number;
}

function required(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(
      `Missing required environment variable ${name}. See .env.example at the repository root.`,
    );
  }
  return value;
}

/**
 * Read configuration, failing immediately if anything is missing.
 *
 * Failing at startup rather than at first use means a misconfigured deploy
 * dies visibly instead of running for an hour and then throwing inside a
 * handler, where the only symptom would be a dashboard that quietly stopped
 * updating.
 */
export function loadConfig(): IndexerConfig {
  return {
    rpcUrl: required("SOROBAN_RPC_URL"),
    networkPassphrase: required("STELLAR_NETWORK_PASSPHRASE"),
    factoryContractId: required("FACTORY_CONTRACT_ID"),
    supabaseUrl: required("SUPABASE_URL"),
    supabaseServiceKey: required("SUPABASE_SERVICE_ROLE_KEY"),
    pollIntervalMs: Number(process.env.INDEXER_POLL_INTERVAL_MS ?? "5000"),
  };
}
