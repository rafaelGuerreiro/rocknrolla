import { DbConnection, tables } from './module_bindings';

const URI = import.meta.env.VITE_SPACETIMEDB_URI ?? 'ws://localhost:3000';
const DATABASE = import.meta.env.VITE_SPACETIMEDB_DB ?? 'rocknrolladb-dev';
// Namespaced by target so a token from one server (e.g. local) is never sent to another (e.g. Maincloud).
const TOKEN_KEY = `rocknrolla_token:${URI}:${DATABASE}`;

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
            tables.vw_level,
            tables.vw_level_layer,
            tables.vw_character,
            tables.vw_piece,
            tables.vw_me,
            tables.vw_my_enabled_level,
            tables.vw_my_completed_level,
            tables.vw_my_lootbox,
            tables.vw_my_piece,
            tables.vw_my_unlocked_character,
          ]);
      })
      .onConnectError((_ctx, error) => fail(error))
      .build();
  });
}
