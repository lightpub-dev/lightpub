import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";

import axios from "axios";

axios.defaults.baseURL = import.meta.env["API_URL"] ?? "http://localhost:8000";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
