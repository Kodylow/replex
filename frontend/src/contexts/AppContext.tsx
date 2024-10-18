import React, {
  createContext,
  useReducer,
  ReactNode,
  useEffect,
  useContext,
} from "react";
import { Tab, Screen, APP_ACTION_TYPE, User } from "../types";
import { useToast } from "@/hooks/use-toast";
import { ReceiveContext } from "./ReceiveContext";
import { SendContext } from "./SendContext";

interface AppState {
  activeTab: Tab;
  currentScreen: Screen;
  isLoggedIn: boolean;
  user: User | null;
  error: string | null;
}

export type AppAction =
  | { type: APP_ACTION_TYPE.SET_ACTIVE_TAB; payload: Tab }
  | { type: APP_ACTION_TYPE.SET_CURRENT_SCREEN; payload: Screen }
  | { type: APP_ACTION_TYPE.SET_LOGGED_IN; payload: boolean }
  | { type: APP_ACTION_TYPE.SET_USER; payload: User | null }
  | { type: APP_ACTION_TYPE.SET_ERROR; payload: string | null }
  | { type: "INIT"; payload: AppState }
  | { type: APP_ACTION_TYPE.CLEAR_TRANSACTION_STATES };

export interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}

const defaultState: AppState = {
  activeTab: Tab.Wallet,
  currentScreen: Screen.Home,
  isLoggedIn: false,
  user: null,
  error: null,
};

const makeInitialState = (): AppState => {
  const storedState = localStorage.getItem("appState");
  if (storedState) {
    try {
      return JSON.parse(storedState);
    } catch (error) {
      console.error("Failed to parse stored state:", error);
    }
  }
  return defaultState;
};

function appReducer(state: AppState, action: AppAction): AppState {
  const newState = (() => {
    switch (action.type) {
      case APP_ACTION_TYPE.SET_ACTIVE_TAB:
        return { ...state, activeTab: action.payload };
      case APP_ACTION_TYPE.SET_CURRENT_SCREEN:
        return { ...state, currentScreen: action.payload };
      case APP_ACTION_TYPE.SET_LOGGED_IN:
        return { ...state, isLoggedIn: action.payload };
      case APP_ACTION_TYPE.SET_USER:
        return { ...state, user: action.payload };
      case APP_ACTION_TYPE.SET_ERROR:
        return { ...state, error: action.payload };
      case "INIT":
        return action.payload;
      case APP_ACTION_TYPE.CLEAR_TRANSACTION_STATES:
        return { ...state };
      default:
        return state;
    }
  })();

  // Only save state if it has changed
  if (JSON.stringify(newState) !== JSON.stringify(state)) {
    chrome.storage.local.set({ appState: newState });
    console.log("App state saved to Chrome storage:", newState);
  }
  return newState;
}

export const AppContext = createContext<AppContextValue>({
  state: defaultState,
  dispatch: () => {},
});

export const AppProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  const [state, dispatch] = useReducer(appReducer, makeInitialState());
  const { toast } = useToast();
  const receiveContext = useContext(ReceiveContext);
  const sendContext = useContext(SendContext);

  useEffect(() => {
    chrome.storage.local.get(["appState"], (result) => {
      if (result.appState) {
        dispatch({ type: "INIT", payload: result.appState });
      }
    });
  }, []);

  useEffect(() => {
    if (state.error) {
      toast({
        title: "Error",
        description: state.error,
        variant: "destructive",
      });
      // Clear the error after showing the toast
      dispatch({ type: APP_ACTION_TYPE.SET_ERROR, payload: null });
    }
  }, [state.error, toast]);

  useEffect(() => {
    if (
      state.currentScreen === Screen.Home ||
      (state.currentScreen !== Screen.Receive &&
        state.currentScreen !== Screen.Send)
    ) {
      dispatch({ type: APP_ACTION_TYPE.CLEAR_TRANSACTION_STATES });
      receiveContext.resetState();
      sendContext.resetState();
    }
  }, [state.currentScreen, receiveContext, sendContext]);

  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
};
