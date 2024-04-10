import React from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Timeline } from "./routes/Timeline";

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
  ],
  {
    basename: "/web",
  }
);

export default function App() {
  return (
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
}
