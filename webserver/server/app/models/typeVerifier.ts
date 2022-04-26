import type Zapatos from 'zapatos/schema';
import { expectType } from 'tsd';
import type { BlockInfo } from '../../../shared/models/TransactionHistory';

// tsoa can't support looking up Zapatos types, so instead we just make sure the types match
expectType<Equals<BlockInfo['era'], Zapatos.Block.Selectable['era']>>(true);
