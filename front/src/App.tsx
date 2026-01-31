import { CssBaseline, ThemeProvider, createTheme } from "@mui/material";
import { AuthProvider, LoginPage, useAuth } from "./auth";
import { UsersPage } from "./users";

const theme = createTheme({
  palette: {
    mode: "light",
  },
});

const AuthenticatedApp = () => {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return null; // LoginPage handles loading state
  }

  return isAuthenticated ? <UsersPage /> : <LoginPage />;
};

export const App = () => (
  <ThemeProvider theme={theme}>
    <CssBaseline />
    <AuthProvider>
      <AuthenticatedApp />
    </AuthProvider>
  </ThemeProvider>
);
