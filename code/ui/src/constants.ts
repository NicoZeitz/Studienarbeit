export const API_URL = import.meta.env.PROD
  ? `${window.location.origin}/api`
  : "http://localhost:3000/api";
