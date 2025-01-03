import { useTranslation } from 'react-i18next';
import { useWalletStore } from '@app/store/useWalletStore';
import { CircularProgress } from '@app/components/elements/CircularProgress';

import { ListLabel } from './HistoryItem.styles';
import { HistoryContainer, HistoryPadding } from './Wallet.styles';
import HistoryItem from './HistoryItem';

const container = {
    hidden: { opacity: 0, height: 0 },
    visible: {
        opacity: 1,
        height: 326,
    },
};

export default function History() {
    const { t } = useTranslation('sidebar', { useSuspense: false });
    const isTransactionLoading = useWalletStore((s) => s.isTransactionLoading);
    const transactions = useWalletStore((s) => s.transactions);

    return (
        <HistoryContainer initial="hidden" animate="visible" exit="hidden" variants={container}>
            <HistoryPadding>
                <ListLabel>{t('recent-wins')}</ListLabel>
                {isTransactionLoading && !transactions?.length ? (
                    <CircularProgress />
                ) : (
                    transactions.map((tx) => <HistoryItem key={tx.tx_id} item={tx} />)
                )}
            </HistoryPadding>
        </HistoryContainer>
    );
}
