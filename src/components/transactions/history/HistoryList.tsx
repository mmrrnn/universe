import { memo, useCallback, useEffect, useMemo } from 'react';
import InfiniteScroll from 'react-infinite-scroll-component';
import { useWalletStore } from '@app/store/useWalletStore';
import { CircularProgress } from '@app/components/elements/CircularProgress';
import { ListItemWrapper, ListWrapper } from './TxHistory.styles.ts';
import { HistoryListItem } from './ListItem.tsx';
import { fetchTransactions, initialFetchTxs } from '@app/store';
import { useTranslation } from 'react-i18next';

interface HistoryListProps {
    winsOnly?: boolean;
}
const HistoryList = ({ winsOnly = false }: HistoryListProps) => {
    const { t } = useTranslation('wallet', { useSuspense: false });
    const is_transactions_history_loading = useWalletStore((s) => s.is_transactions_history_loading);
    const transactions = useWalletStore((s) => s.transactions);
    const hasMore = useWalletStore((s) => s.has_more_transactions);

    useEffect(() => {
        initialFetchTxs();
    }, []);

    const lastTxId = useMemo(() => transactions[transactions.length - 1].tx_id, [transactions]);

    const handleNext = useCallback(async () => {
        if (!is_transactions_history_loading) {
            await fetchTransactions({ lastTxId, limit: 20 });
        }
    }, [is_transactions_history_loading, lastTxId]);

    return (
        <ListWrapper id="list">
            {!is_transactions_history_loading && !transactions?.length && t('empty-tx')}
            <InfiniteScroll
                dataLength={transactions?.length || 0}
                next={handleNext}
                hasMore={hasMore}
                loader={<CircularProgress />}
                scrollableTarget="list"
            >
                <ListItemWrapper>
                    {transactions.map((tx, index) => (
                        <HistoryListItem key={tx.tx_id} item={tx} index={index} showReplay={winsOnly} />
                    ))}
                </ListItemWrapper>
            </InfiniteScroll>
        </ListWrapper>
    );
};

export default memo(HistoryList);
