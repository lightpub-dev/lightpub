import "./App.css";
import { ChakraProvider } from "@chakra-ui/react";
import { store } from "./store";
import { Provider } from "react-redux";
import TimelinePage from "./pages/timeline/TimelinePage.tsx";
import ProfilePage from "./pages/user/ProfilePage.tsx";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import ErrorPage from "./ErrorPage.tsx";
import LoginPage from "./components/auth/LoginPage.tsx";
import RegistrationPage from "./components/auth/RegistrationPage.tsx";

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

export function App() {
  return (
    <ChakraProvider>
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>
    </ChakraProvider>
  );
}

export default App;
