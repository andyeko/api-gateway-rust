import { CssBaseline, ThemeProvider, createTheme } from "@mui/material";
import { UsersPage } from "./users";

const theme = createTheme({
  palette: {
    mode: "light"
  }
});

export const App = () => (
  <ThemeProvider theme={theme}>
    <CssBaseline />
    <UsersPage />
  </ThemeProvider>
);
