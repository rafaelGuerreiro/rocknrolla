import { Uuid } from 'spacetimedb';
import { db } from './db';

/** How long to wait for the server to open a lootbox. */
const OPEN_TIMEOUT_MS = 6000;

type LootboxRow = { id: Uuid; opened: boolean; awardedPieceId?: Uuid };

/**
 * Ask the server to open a lootbox and resolve with the awarded piece id.
 * The client never chooses the piece; rejection covers reducer errors and
 * a confirmation timeout. `vw_my_lootbox` is a computed view without a
 * primary key, so the opened row arrives as a delete + insert — listen on
 * inserts (plus updates, in case the SDK ever diffs by value).
 */
export function openLootboxAndAwaitPiece(playerLootboxId: Uuid): Promise<Uuid> {
  const conn = db();
  return new Promise((resolve, reject) => {
    const settle = (fn: () => void) => {
      conn.db.vw_my_lootbox.removeOnInsert(onInsert);
      conn.db.vw_my_lootbox.removeOnUpdate(onUpdate);
      clearTimeout(timer);
      fn();
    };
    const check = (row: LootboxRow) => {
      const piece = row.awardedPieceId;
      if (row.id.compareTo(playerLootboxId) !== 0 || !row.opened || !piece)
        return;
      settle(() => resolve(piece));
    };
    const onInsert = (_ctx: unknown, row: LootboxRow) => check(row);
    const onUpdate = (_ctx: unknown, _old: LootboxRow, row: LootboxRow) =>
      check(row);
    const timer = setTimeout(
      () =>
        settle(() =>
          reject(new Error('The server did not answer. Check the connection.')),
        ),
      OPEN_TIMEOUT_MS,
    );
    conn.db.vw_my_lootbox.onInsert(onInsert);
    conn.db.vw_my_lootbox.onUpdate(onUpdate);
    conn.reducers.openLootbox({ playerLootboxId }).catch((error: unknown) => {
      settle(() =>
        reject(error instanceof Error ? error : new Error(String(error))),
      );
    });
  });
}
