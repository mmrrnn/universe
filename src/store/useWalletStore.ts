import { create } from './create';
import { WalletAddress, TransactionInfo, WalletBalance, TxType, TransactionStatus } from '../types/app-status.ts';
import { invoke } from '@tauri-apps/api/core';
import { ALREADY_FETCHING } from '@app/App/sentryIgnore.ts';

interface State {
    tari_address_base58: string;
    tari_address_emoji: string;
    balance?: WalletBalance;
    calculated_balance?: number;
    coinbase_transactions: TransactionInfo[];
    transactions: TransactionInfo[];
    is_reward_history_loading: boolean;
    has_more_coinbase_transactions: boolean;
    has_more_transactions: boolean;
    is_transactions_history_loading: boolean;
    is_wallet_importing: boolean;
}

interface Actions {
    setWalletAddress: (wallet_address: WalletAddress) => void;
    setWalletBalance: (wallet_balance: WalletBalance) => void;
    importSeedWords: (seedWords: string[]) => Promise<void>;
    fetchCoinbaseTransactions: (lastTxId?: number, limit?: number) => Promise<TransactionInfo[]>;
    refreshCoinbaseTransactions: () => Promise<TransactionInfo[]>;
    fetchTransactions: (p?: {
        lastTxId?: number;
        statusFilters?: TransactionStatus[];
        limit?: number;
    }) => Promise<TransactionInfo[]>;
    refreshTransactions: () => Promise<TransactionInfo[]>;
}

type WalletStoreState = State & Actions;

const initialState: State = {
    tari_address_base58: '',
    tari_address_emoji: '',
    coinbase_transactions: [],
    transactions: [],
    has_more_coinbase_transactions: true,
    has_more_transactions: true,
    is_reward_history_loading: false,
    is_transactions_history_loading: false,
    is_wallet_importing: false,
};

export const useWalletStore = create<WalletStoreState>()((set, getState) => ({
    ...initialState,
    setWalletAddress: (wallet_address) => {
        set({ ...wallet_address });
    },
    setWalletBalance: (balance) => {
        const calculated_balance =
            balance.available_balance + balance.timelocked_balance + balance.pending_incoming_balance;
        set({ balance, calculated_balance });
    },
    fetchTransactions: async ({ lastTxId, statusFilters, limit } = {}) => {
        if (useWalletStore.getState().is_transactions_history_loading) {
            return [];
        }

        try {
            useWalletStore.setState({ is_transactions_history_loading: true });

            const fetchedTxs = await invoke('get_transactions', {
                lastTxId,
                statusFilters,
                limit,
            });

            const txWithType = fetchedTxs.map((tx: TransactionInfo) => ({
                ...tx,
                txType: (tx.direction === 2
                    ? 'sent'
                    : !tx.mined_in_block_height || tx.mined_in_block_height === 0
                      ? 'received'
                      : 'mined') as TxType,
            }));

            const transactions = lastTxId ? [...getState().transactions, ...txWithType] : txWithType;
            const has_more_transactions = txWithType.length > 0 && (!limit || txWithType.length === limit);

            set({
                has_more_transactions,
                transactions,
            });
            return transactions;
        } catch (error) {
            if (error !== ALREADY_FETCHING.HISTORY) {
                console.error('Could not get transaction history: ', error);
            }
            return [];
        } finally {
            useWalletStore.setState({ is_transactions_history_loading: false });
        }
    },
    refreshTransactions: async () => {
        const limit = getState().transactions.length;
        return getState().fetchTransactions({ limit: Math.max(limit, 20) });
    },
    fetchCoinbaseTransactions: async (lastTxId, limit) => {
        if (useWalletStore.getState().is_reward_history_loading) {
            return [];
        }

        try {
            useWalletStore.setState({ is_reward_history_loading: true });

            const fetchedTxs = await invoke('get_transactions', {
                lastTxId,
                statusFilters: [TransactionStatus.COINBASE_UNCONFIRMED, TransactionStatus.COINBASE_CONFIRMED],
                limit,
            });

            const coinbase_transactions = lastTxId ? [...getState().coinbase_transactions, ...fetchedTxs] : fetchedTxs;
            const has_more_coinbase_transactions = fetchedTxs.length > 0 && (!limit || fetchedTxs.length === limit);

            set({
                has_more_coinbase_transactions,
                coinbase_transactions,
            });
            return coinbase_transactions;
        } catch (error) {
            if (error !== ALREADY_FETCHING.HISTORY) {
                console.error('Could not get transaction history: ', error);
            }
            return [];
        } finally {
            useWalletStore.setState({ is_reward_history_loading: false });
        }
    },
    refreshCoinbaseTransactions: async () => {
        const limit = getState().coinbase_transactions.length;
        return getState().fetchCoinbaseTransactions(undefined, limit);
    },
    importSeedWords: async (seedWords: string[]) => {
        try {
            set({ is_wallet_importing: true });
            await invoke('import_seed_words', { seedWords });
        } catch (error) {
            console.error('Could not import seed words: ', error);
        }
    },
}));

export const initialFetchTxs = () => {
    useWalletStore.getState().fetchTransactions({ limit: 20 });
};
