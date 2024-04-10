import React, { createContext, useCallback, useContext, useState } from "react";
import axios from "axios";
import { API_URL } from "./settings";

axios.defaults.baseURL = API_URL;

export const RequestContext = createContext<Requester>(
  null as unknown as Requester // DO NOT USE THIS VALUE
);

export const useRequestContext = () => useContext(RequestContext);

export type Requester = ReturnType<typeof createRequester>;

export function createRequester() {
  const [bearerToken, setBearerToken] = useState<string | null>(null);

  const mergeHeaders = useCallback(
    <T extends object>(headers?: T) => {
      if (bearerToken === null) return headers;

      return {
        ...(headers || {}),
        Authorization: `Bearer ${bearerToken}`,
      } as T & { Authorization: string };
    },
    [bearerToken]
  );

  const get = useCallback(
    (url: string, headers?: object) => {
      return axios.get(url, {
        headers: mergeHeaders(headers),
      });
    },
    [mergeHeaders]
  );

  const post = useCallback(
    (url: string, data: any, headers?: object) => {
      return axios.post(url, data, {
        headers: mergeHeaders(headers),
      });
    },
    [mergeHeaders]
  );

  const put = useCallback(
    (url: string, data: any, headers?: object) => {
      return axios.put(url, data, {
        headers: mergeHeaders(headers),
      });
    },
    [mergeHeaders]
  );

  const deleteMethod = useCallback(
    (url: string, headers?: object) => {
      return axios.delete(url, {
        headers: mergeHeaders(headers),
      });
    },
    [mergeHeaders]
  );

  const patch = useCallback(
    (url: string, data: any, headers?: object) => {
      return axios.patch(url, data, {
        headers: mergeHeaders(headers),
      });
    },
    [mergeHeaders]
  );

  return {
    setBearerToken,
    get,
    post,
    put,
    delete: deleteMethod,
    patch,
  };
}
