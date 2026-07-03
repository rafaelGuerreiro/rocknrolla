import { DbConnection, tables } from './module_bindings';

const URI = import.meta.env.VITE_SPACETIMEDB_URI ?? 'ws://localhost:3000';
const DATABASE = import.meta.env.VITE_SPACETIMEDB_DB ?? 'rocknrolladb-dev';
const TOKEN_KEY = 'rocknrolla_token';

let conn: DbConnection | null = null;

/** The active connection. Only valid after `connect()` resolves. */
export function db(): DbConnection {
  if (!conn) throw new Error('not connected');
  return conn;
}

function savedToken(): string | undefined {
  try {
    return localStorage.getItem(TOKEN_KEY) ?? undefined;
  } catch {
    return undefined;
  }
}

/**
 * Connect, persist the session token, and subscribe to the content and
 * caller-owned state the scenes need. Resolves once the subscription applies.
 */
export function connect(): Promise<DbConnection> {
  if (conn) return Promise.resolve(conn);
  return new Promise((resolve, reject) => {
    const fail = (error: unknown) => {
      conn = null;
      reject(error instanceof Error ? error : new Error(String(error)));
    };
    conn = DbConnection.builder()
      .withUri(URI)
      .withDatabaseName(DATABASE)
      .withToken(savedToken())
      .onConnect((connection, _identity, token) => {
        try {
          localStorage.setItem(TOKEN_KEY, token);
        } catch {
          // Private browsing: play without a persistent identity.
        }
        connection
          .subscriptionBuilder()
          .onApplied(() => resolve(connection))
          .onError((ctx) => fail(ctx.event))
          .subscribe([
            tables.level,
            tables.level_layer,
            tables.character_def,
            tables.piece_def,
            tables.lootbox_def,
            tables.player,
            tables.player_enabled_level,
            tables.player_completed_level,
            tables.player_lootbox,
            tables.player_piece,
            tables.player_unlocked_character,
          ]);
      })
      .onConnectError((_ctx, error) => fail(error))
      .build();
  });
}
