import { useDispatch, useSelector } from "react-redux";
import type { AppDispatch, RootState } from "./store";
import axios from "axios";

// Use throughout your app instead of plain `useDispatch` and `useSelector`
export const useAppDispatch = useDispatch.withTypes<AppDispatch>();
export const useAppSelector = useSelector.withTypes<RootState>();

export const authedFetcher = <T>([authorization, url]: [
  string,
  string,
]): Promise<T> =>
  axios
    .get(url, {
      headers: {
        authorization,
      },
    })
    .then((r) => r.data);

export const cookieFetcher = <T>(url: string): Promise<T> =>
  axios
    .get(url, {
      withCredentials: true,
    })
    .then((r) => r.data);
