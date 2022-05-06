import type Zapatos from 'zapatos/schema';
import { expectType } from 'tsd';
import type { BlockInfo } from '../../../shared/models/TransactionHistory';
import type { BlockLatestResponse } from '../../../shared/models/BlockLatest';

// tsoa can't support looking up Zapatos types, so instead we just make sure the types match
expectType<Equals<BlockInfo['era'], Zapatos.Block.Selectable['era']>>(true);
expectType<
  Equals<
    WithoutDatabaseId<BuffersToStrings<Zapatos.Block.Selectable>>,
    BlockLatestResponse['block']
  >
>(true);
