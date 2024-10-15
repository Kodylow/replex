import React, { createContext, useReducer } from "react";
import { FedimintWallet } from "@fedimint/core-web";
import { WALLET_ACTION_TYPE } from "@/types";
import { wallet } from "@/wallet";
export interface WalletState {
  wallet: FedimintWallet;
  error: string | null;
}

export type WalletAction = {
  type: WALLET_ACTION_TYPE.SET_ERROR;
  payload: string;
};

export interface WalletContextValue {
  state: WalletState;
  dispatch: React.Dispatch<WalletAction>;
}

const initialState: WalletState = {
  wallet: wallet,
  error: null,
};

function walletReducer(state: WalletState, action: WalletAction): WalletState {
  switch (action.type) {
    case WALLET_ACTION_TYPE.SET_ERROR:
      return { ...state, error: action.payload };
    default:
      return state;
  }
}

export const WalletContext = createContext<WalletContextValue>({
  state: initialState,
  dispatch: () => {},
});

export const WalletProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [state, dispatch] = useReducer(walletReducer, initialState);

  if (!state.wallet) {
    // Render a loading state or null while the wallet is being initialized
    return null;
  }

  const contextValue = {
    state,
    dispatch,
  };

  return (
    <WalletContext.Provider value={contextValue}>
      {children}
    </WalletContext.Provider>
  );
};
