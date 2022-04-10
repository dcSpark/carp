import prisma from './PrismaSingleton';

export async function countTxs() {
  const numTxs = await prisma.transaction.count();
  return numTxs;
}
