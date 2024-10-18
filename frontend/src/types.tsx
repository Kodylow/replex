export enum Screen {
  Home = "home",
  Send = "send",
  Receive = "receive",
}

export enum Tab {
  Wallet = "wallet",
  Settings = "settings",
}

export interface User {
  id: string;
  name: string;
}

export enum APP_ACTION_TYPE {
  SET_ACTIVE_TAB = "SET_ACTIVE_TAB",
  SET_CURRENT_SCREEN = "SET_CURRENT_SCREEN",
  SET_LOGGED_IN = "SET_LOGGED_IN",
  SET_USER = "SET_USER",
  SET_ERROR = "SET_ERROR",
}

export enum WALLET_ACTION_TYPE {
  SET_ERROR = "SET_ERROR",
  ADD_TRANSACTION = "ADD_TRANSACTION",
  UPDATE_TRANSACTION = "UPDATE_TRANSACTION",
}
