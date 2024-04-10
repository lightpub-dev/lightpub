import React, { useEffect, useRef } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Timeline } from "./routes/Timeline";
import { RegisterView } from "./routes/Register";
import { createRequester, RequestContext, Requester } from "./requester";

const router = createBrowserRouter(
  [
    {
      path: "/ping",
      element: <div>Pong!</div>,
    },
    {
      path: "/timeline",
      element: <Timeline />,
    },
    {
      path: "/",
      element: <div>Root</div>,
    },
    {
      path: "/register",
      element: <RegisterView />,
    },
  ],
  {
    basename: "/web",
  }
);

export default function App() {
  const requester = createRequester();

  return (
    <RequestContext.Provider value={requester}>
      <RouterProvider router={router} />
    </RequestContext.Provider>
  );
}
