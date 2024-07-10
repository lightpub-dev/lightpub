import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import ErrorPage from "./ErrorPage.tsx";
import LoginPage from "./components/auth/LoginPage.tsx";
import RegistrationPage from "./components/auth/RegistrationPage.tsx";
import axios from "axios";
import { ChakraProvider } from "@chakra-ui/react";
import { store } from "./store";
import { Provider } from "react-redux";
import TimelinePage from "./pages/timeline/TimelinePage.tsx";
import ProfilePage from "./pages/user/ProfilePage.tsx";

axios.defaults.baseURL = import.meta.env["API_URL"] ?? "http://localhost:8000";

const router = createBrowserRouter([
  {
    path: "/",
    element: <TimelinePage />,
    errorElement: <ErrorPage />,
  },
  {
    path: "/login",
    element: <LoginPage />,
  },
  {
    path: "/user/:userId",
    element: <ProfilePage />,
  },
  {
    path: "/register",
    element: <RegistrationPage />,
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ChakraProvider>
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>
    </ChakraProvider>
  </React.StrictMode>
);
