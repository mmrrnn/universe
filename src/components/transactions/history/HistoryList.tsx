import { memo, useCallback, useEffect } from 'react';
import InfiniteScroll from 'react-infinite-scroll-component';
import { useWalletStore } from '@app/store/useWalletStore';
import { CircularProgress } from '@app/components/elements/CircularProgress';
import { ListItemWrapper, ListWrapper } from './TxHistory.styles.ts';
import { HistoryListItem } from './ListItem.tsx';
import { initialFetchTxs, fetchTransactionsHistory } from '@app/store';
import { useTranslation } from 'react-i18next';
import { Typography } from '@app/components/elements/Typography.tsx';
import { TransactionInfo } from '@app/types/app-status.ts';

const HistoryList = () => {
    const { t } = useTranslation('wallet', { useSuspense: false });
    const is_transactions_history_loading = useWalletStore((s) => s.is_transactions_history_loading);
    const transactions = useWalletStore((s) => s.transactions);
    const pendingTransactions = useWalletStore((s) => s.pending_transactions);
    const hasMore = useWalletStore((s) => s.has_more_transactions);
    const walletScanning = useWalletStore((s) => s.wallet_scanning);

    useEffect(() => {
        initialFetchTxs();
    }, []);

    const combinedTransactions = [...pendingTransactions, ...transactions] as TransactionInfo[];

    const handleNext = useCallback(async () => {
        if (!is_transactions_history_loading) {
            await fetchTransactionsHistory({ continuation: true, limit: 20 });
        }
    }, [is_transactions_history_loading]);

    return (
        <ListWrapper id="list">
            {!walletScanning.is_scanning && !is_transactions_history_loading && !combinedTransactions?.length && (
                <Typography variant="h6">{t('empty-tx')}</Typography>
            )}
            {walletScanning.is_scanning ? (
                <Typography variant="h6" style={{ textAlign: 'left' }}>
                    {walletScanning.is_scanning && walletScanning.total_height > 0
                        ? t('wallet-scanning-with-progress', {
                              scanned: walletScanning.scanned_height.toLocaleString(),
                              total: walletScanning.total_height.toLocaleString(),
                              percent: walletScanning.progress.toFixed(1),
                          })
                        : t('wallet-is-scanning')}
                </Typography>
            ) : (
                <InfiniteScroll
                    dataLength={combinedTransactions?.length || 0}
                    next={handleNext}
                    hasMore={hasMore}
                    loader={<CircularProgress />}
                    scrollableTarget="list"
                >
                    <ListItemWrapper>
                        {combinedTransactions.map((tx, index) => (
                            <HistoryListItem key={tx.tx_id} item={tx} index={index} />
                        ))}
                    </ListItemWrapper>
                </InfiniteScroll>
            )}
        </ListWrapper>
    );
};

export default memo(HistoryList);
