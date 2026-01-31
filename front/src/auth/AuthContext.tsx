import React, { createContext, useContext, useEffect, useState, useCallback } from "react";

const AUTH_API_BASE = import.meta.env.VITE_AUTH_API_BASE ?? "http://localhost:4002/auth";

export type User = {
  id: string;
  email: string;
  name: string;
};

export type AuthState = {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
};

type AuthContextType = AuthState & {
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  register: (email: string, name: string, password: string) => Promise<void>;
};

const AuthContext = createContext<AuthContextType | null>(null);

const STORAGE_KEY = "apisentinel_auth";

type StoredAuth = {
  user: User;
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
};

function loadStoredAuth(): StoredAuth | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return null;
    return JSON.parse(stored) as StoredAuth;
  } catch {
    return null;
  }
}

function saveAuth(user: User, accessToken: string, refreshToken: string, expiresIn: number) {
  const expiresAt = Date.now() + expiresIn * 1000;
  const data: StoredAuth = { user, accessToken, refreshToken, expiresAt };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
}

function clearAuth() {
  localStorage.removeItem(STORAGE_KEY);
}

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, setState] = useState<AuthState>({
    user: null,
    accessToken: null,
    refreshToken: null,
    isAuthenticated: false,
    isLoading: true,
  });

  // Refresh token function
  const refreshAccessToken = useCallback(async (refreshToken: string): Promise<boolean> => {
    try {
      const response = await fetch(`${AUTH_API_BASE}/refresh`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: refreshToken }),
      });

      if (!response.ok) {
        return false;
      }

      const data = await response.json();
      saveAuth(data.user, data.access_token, data.refresh_token, data.expires_in);
      setState({
        user: data.user,
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        isAuthenticated: true,
        isLoading: false,
      });
      return true;
    } catch {
      return false;
    }
  }, []);

  // Initialize auth state from storage
  useEffect(() => {
    const stored = loadStoredAuth();
    if (!stored) {
      setState(prev => ({ ...prev, isLoading: false }));
      return;
    }

    // Check if token is expired or about to expire (within 1 minute)
    const now = Date.now();
    const expiresIn = stored.expiresAt - now;

    if (expiresIn < 60000) {
      // Token expired or expiring soon, try to refresh
      refreshAccessToken(stored.refreshToken).then(success => {
        if (!success) {
          clearAuth();
          setState({
            user: null,
            accessToken: null,
            refreshToken: null,
            isAuthenticated: false,
            isLoading: false,
          });
        }
      });
    } else {
      // Token is still valid
      setState({
        user: stored.user,
        accessToken: stored.accessToken,
        refreshToken: stored.refreshToken,
        isAuthenticated: true,
        isLoading: false,
      });
    }
  }, [refreshAccessToken]);

  // Set up automatic token refresh (every minute)
  useEffect(() => {
    if (!state.refreshToken) return;

    const interval = setInterval(() => {
      const stored = loadStoredAuth();
      if (!stored) return;

      const now = Date.now();
      const expiresIn = stored.expiresAt - now;

      // Refresh if token expires within 2 minutes
      if (expiresIn < 120000) {
        refreshAccessToken(stored.refreshToken);
      }
    }, 60000); // Check every minute

    return () => clearInterval(interval);
  }, [state.refreshToken, refreshAccessToken]);

  const login = async (email: string, password: string) => {
    const response = await fetch(`${AUTH_API_BASE}/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || "Login failed");
    }

    const data = await response.json();
    saveAuth(data.user, data.access_token, data.refresh_token, data.expires_in);
    setState({
      user: data.user,
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      isAuthenticated: true,
      isLoading: false,
    });
  };

  const logout = async () => {
    if (state.refreshToken) {
      try {
        await fetch(`${AUTH_API_BASE}/logout`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ refresh_token: state.refreshToken }),
        });
      } catch {
        // Ignore logout errors
      }
    }

    clearAuth();
    setState({
      user: null,
      accessToken: null,
      refreshToken: null,
      isAuthenticated: false,
      isLoading: false,
    });
  };

  const register = async (email: string, name: string, password: string) => {
    const response = await fetch(`${AUTH_API_BASE}/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, name, password }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || "Registration failed");
    }

    const data = await response.json();
    saveAuth(data.user, data.access_token, data.refresh_token, data.expires_in);
    setState({
      user: data.user,
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      isAuthenticated: true,
      isLoading: false,
    });
  };

  return (
    <AuthContext.Provider value={{ ...state, login, logout, register }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
