import React, { useEffect, useRef } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Timeline } from "./routes/Timeline";
import { RegisterView } from "./routes/Register";
import { createRequester, RequestContext, Requester } from "./requester";
import { LoginView } from "./routes/Login";
import { UserProfile } from "./routes/UserProfile";
import { loader as userProfileLoader } from "./routes/UserProfile";

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
    {
      path: "/login",
      element: <LoginView />,
    },
    {
      path: "/user/:userId",
      element: <UserProfile />,
      loader: userProfileLoader as any,
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
