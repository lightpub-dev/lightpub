if (!process.env.API_URL) {
  console.warn("API_URL is not defined. Falling back to http://localhost:1234");
}
export const API_URL = process.env.API_URL || "http://localhost:1234";
