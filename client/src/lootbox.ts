import { Uuid } from 'spacetimedb';
import { db } from './db';

/** How long to wait for the server to open a lootbox. */
const OPEN_TIMEOUT_MS = 6000;

type LootboxRow = { id: Uuid; opened: boolean; awardedPieceId?: Uuid };

/**
 * Ask the server to open a lootbox and resolve with the awarded piece id.
 * The client never chooses the piece; rejection covers reducer errors and
 * a confirmation timeout. Used by the result reveal and the collection.
 */
export function openLootboxAndAwaitPiece(playerLootboxId: Uuid): Promise<Uuid> {
  const conn = db();
  return new Promise((resolve, reject) => {
    const settle = (fn: () => void) => {
      conn.db.vw_my_lootbox.removeOnUpdate(onUpdate);
      clearTimeout(timer);
      fn();
    };
    const onUpdate = (_ctx: unknown, _old: LootboxRow, row: LootboxRow) => {
      const piece = row.awardedPieceId;
      if (row.id.compareTo(playerLootboxId) !== 0 || !row.opened || !piece)
        return;
      settle(() => resolve(piece));
    };
    const timer = setTimeout(
      () =>
        settle(() =>
          reject(new Error('The server did not answer. Check the connection.')),
        ),
      OPEN_TIMEOUT_MS,
    );
    conn.db.vw_my_lootbox.onUpdate(onUpdate);
    conn.reducers.openLootbox({ playerLootboxId }).catch((error: unknown) => {
      settle(() =>
        reject(error instanceof Error ? error : new Error(String(error))),
      );
    });
  });
}
