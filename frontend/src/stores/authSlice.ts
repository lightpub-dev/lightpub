import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import type { RootState } from "../store";

export interface AuthState {
  token: string | null;
  username: string | null;
}

const initialState: AuthState = {
  token: null,
  username: null,
};

export const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {
    loggedIn: (
      state,
      action: PayloadAction<{ token: string; username: string }>
    ) => {
      state.token = action.payload.token;
      state.username = action.payload.username;
    },
    logout: (state) => {
      state.token = null;
      state.username = null;
    },
  },
});

export const { loggedIn, logout } = authSlice.actions;
export const selectToken = (state: RootState) => state.auth.token;
export const selectUsername = (state: RootState) => state.auth.username;
export const selectIsLoggedIn = (state: RootState) => state.auth.token !== null;
export const selectAuthorization = (state: RootState) =>
  `Bearer ${state.auth.token}`;

export default authSlice.reducer;
