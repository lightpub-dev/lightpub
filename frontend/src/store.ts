import { configureStore } from "@reduxjs/toolkit";
import authReducer from "./stores/authSlice";
import { persistReducer, persistStore } from "redux-persist";
import storage from "redux-persist/lib/storage";

const authPersistConfig = {
  key: "auth",
  storage,
};

export const store = configureStore({
  reducer: { auth: persistReducer(authPersistConfig, authReducer) },
});
export const persistor = persistStore(store);

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
