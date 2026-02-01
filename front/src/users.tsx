import { useEffect, useMemo, useState } from "react";
import {
  Alert,
  AppBar,
  Box,
  Button,
  Chip,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TextField,
  Toolbar,
  Typography
} from "@mui/material";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import LogoutIcon from "@mui/icons-material/Logout";
import { useAuth } from "./auth";

type Role = "SUPER_ADMIN" | "ADMIN" | "SUPERVISOR" | "USER";

type User = {
  id: string;
  organisation_id: string | null;
  email: string;
  name: string;
  role: Role;
  created_at: string;
  updated_at: string;
};

type Organisation = {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
};

type UserInput = {
  email: string;
  name: string;
  password: string;
  role: Role;
  organisation_id: string | null;
};

const API_BASE = import.meta.env.VITE_API_BASE ?? "/admin";
const API_KEY = import.meta.env.VITE_API_KEY ?? "dev";

const ROLES: Role[] = ["SUPER_ADMIN", "ADMIN", "SUPERVISOR", "USER"];

const roleColor = (role: Role): "error" | "warning" | "info" | "default" => {
  switch (role) {
    case "SUPER_ADMIN": return "error";
    case "ADMIN": return "warning";
    case "SUPERVISOR": return "info";
    default: return "default";
  }
};

export const UsersPage = () => {
  const { logout, user } = useAuth();
  const [users, setUsers] = useState<User[]>([]);
  const [organisations, setOrganisations] = useState<Organisation[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [form, setForm] = useState<UserInput>({ email: "", name: "", password: "", role: "USER", organisation_id: null });
  const [editId, setEditId] = useState<string | null>(null);

  const canSubmit = useMemo(() => form.email.trim() && form.name.trim(), [form]);

  const loadUsers = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE}/users?_start=0&_end=100`, {
        headers: { Accept: "application/json", "x-api-key": API_KEY }
      });
      if (!response.ok) {
        throw new Error(`Failed to load users: ${response.status}`);
      }
      const data = (await response.json()) as User[];
      setUsers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  };

  const loadOrganisations = async () => {
    try {
      const response = await fetch(`${API_BASE}/organisations?_start=0&_end=100`, {
        headers: { Accept: "application/json", "x-api-key": API_KEY }
      });
      if (response.ok) {
        const data = (await response.json()) as Organisation[];
        setOrganisations(data);
      }
    } catch {
      // Organisations are optional, ignore errors
    }
  };

  useEffect(() => {
    loadUsers();
    loadOrganisations();
  }, []);

  const resetForm = () => setForm({ email: "", name: "", password: "", role: "USER", organisation_id: null });

  const openCreate = () => {
    resetForm();
    setCreateOpen(true);
  };

  const openEdit = (user: User) => {
    setForm({ 
      email: user.email, 
      name: user.name, 
      password: "", 
      role: user.role,
      organisation_id: user.organisation_id 
    });
    setEditId(user.id);
    setEditOpen(true);
  };

  const handleCreate = async () => {
    if (!canSubmit) return;
    setLoading(true);
    setError(null);
    try {
      const payload: { email: string; name: string; password?: string; role: Role; organisation_id?: string } = {
        email: form.email,
        name: form.name,
        role: form.role,
      };
      if (form.password.trim()) {
        payload.password = form.password;
      }
      if (form.organisation_id) {
        payload.organisation_id = form.organisation_id;
      }
      const response = await fetch(`${API_BASE}/users`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "x-api-key": API_KEY },
        body: JSON.stringify(payload)
      });
      if (!response.ok) {
        throw new Error(`Failed to create user: ${response.status}`);
      }
      setCreateOpen(false);
      resetForm();
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  };

  const handleUpdate = async () => {
    if (!editId || !canSubmit) return;
    setLoading(true);
    setError(null);
    try {
      const payload: { email: string; name: string; password?: string; role: Role; organisation_id?: string | null } = {
        email: form.email,
        name: form.name,
        role: form.role,
        organisation_id: form.organisation_id,
      };
      if (form.password.trim()) {
        payload.password = form.password;
      }
      const response = await fetch(`${API_BASE}/users/${editId}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json", "x-api-key": API_KEY },
        body: JSON.stringify(payload)
      });
      if (!response.ok) {
        throw new Error(`Failed to update user: ${response.status}`);
      }
      setEditOpen(false);
      setEditId(null);
      resetForm();
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE}/users/${id}`, {
        method: "DELETE",
        headers: { "x-api-key": API_KEY }
      });
      if (!response.ok) {
        throw new Error(`Failed to delete user: ${response.status}`);
      }
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  };

  const getOrgName = (orgId: string | null) => {
    if (!orgId) return "-";
    const org = organisations.find(o => o.id === orgId);
    return org ? org.name : orgId.slice(0, 8) + "...";
  };

  return (
    <>
      <AppBar position="static">
        <Toolbar>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            API Sentinel
          </Typography>
          {user && (
            <Typography variant="body2" sx={{ mr: 2 }}>
              {user.email}
            </Typography>
          )}
          <IconButton color="inherit" onClick={logout} title="Sign out">
            <LogoutIcon />
          </IconButton>
        </Toolbar>
      </AppBar>
      <Container maxWidth="lg" sx={{ py: 4 }}>
      <Stack spacing={2}>
        <Box display="flex" justifyContent="space-between" alignItems="center">
          <Typography variant="h4">Users</Typography>
          <Button variant="contained" onClick={openCreate} disabled={loading}>
            Create user
          </Button>
        </Box>

        {error && <Alert severity="error">{error}</Alert>}

        <Paper variant="outlined">
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Email</TableCell>
                <TableCell>Name</TableCell>
                <TableCell>Role</TableCell>
                <TableCell>Organisation</TableCell>
                <TableCell>Created</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {users.map((user) => (
                <TableRow key={user.id} hover>
                  <TableCell>{user.email}</TableCell>
                  <TableCell>{user.name}</TableCell>
                  <TableCell>
                    <Chip 
                      label={user.role.replace("_", " ")} 
                      size="small" 
                      color={roleColor(user.role)}
                    />
                  </TableCell>
                  <TableCell>{getOrgName(user.organisation_id)}</TableCell>
                  <TableCell>{new Date(user.created_at).toLocaleString()}</TableCell>
                  <TableCell align="right">
                    <IconButton onClick={() => openEdit(user)} size="small">
                      <EditIcon fontSize="small" />
                    </IconButton>
                    <IconButton
                      onClick={() => handleDelete(user.id)}
                      size="small"
                      color="error"
                    >
                      <DeleteIcon fontSize="small" />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
              {!users.length && !loading && (
                <TableRow>
                  <TableCell colSpan={6} align="center">
                    No users yet.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </Paper>
      </Stack>

      <Dialog open={createOpen} onClose={() => setCreateOpen(false)} fullWidth maxWidth="sm">
        <DialogTitle>Create user</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField
              label="Email"
              value={form.email}
              onChange={(event) => setForm({ ...form, email: event.target.value })}
              fullWidth
            />
            <TextField
              label="Name"
              value={form.name}
              onChange={(event) => setForm({ ...form, name: event.target.value })}
              fullWidth
            />
            <TextField
              label="Password"
              type="password"
              value={form.password}
              onChange={(event) => setForm({ ...form, password: event.target.value })}
              fullWidth
              helperText="Optional - leave empty for no password"
            />
            <FormControl fullWidth>
              <InputLabel>Role</InputLabel>
              <Select
                value={form.role}
                label="Role"
                onChange={(event) => setForm({ ...form, role: event.target.value as Role })}
              >
                {ROLES.map((role) => (
                  <MenuItem key={role} value={role}>{role.replace("_", " ")}</MenuItem>
                ))}
              </Select>
            </FormControl>
            <FormControl fullWidth>
              <InputLabel>Organisation</InputLabel>
              <Select
                value={form.organisation_id ?? ""}
                label="Organisation"
                onChange={(event) => setForm({ ...form, organisation_id: event.target.value || null })}
              >
                <MenuItem value="">None</MenuItem>
                {organisations.map((org) => (
                  <MenuItem key={org.id} value={org.id}>{org.name}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={handleCreate} disabled={!canSubmit || loading}>
            Create
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={editOpen} onClose={() => setEditOpen(false)} fullWidth maxWidth="sm">
        <DialogTitle>Edit user</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ mt: 1 }}>
            <TextField
              label="Email"
              value={form.email}
              onChange={(event) => setForm({ ...form, email: event.target.value })}
              fullWidth
            />
            <TextField
              label="Name"
              value={form.name}
              onChange={(event) => setForm({ ...form, name: event.target.value })}
              fullWidth
            />
            <TextField
              label="New Password"
              type="password"
              value={form.password}
              onChange={(event) => setForm({ ...form, password: event.target.value })}
              fullWidth
              helperText="Leave empty to keep current password"
            />
            <FormControl fullWidth>
              <InputLabel>Role</InputLabel>
              <Select
                value={form.role}
                label="Role"
                onChange={(event) => setForm({ ...form, role: event.target.value as Role })}
              >
                {ROLES.map((role) => (
                  <MenuItem key={role} value={role}>{role.replace("_", " ")}</MenuItem>
                ))}
              </Select>
            </FormControl>
            <FormControl fullWidth>
              <InputLabel>Organisation</InputLabel>
              <Select
                value={form.organisation_id ?? ""}
                label="Organisation"
                onChange={(event) => setForm({ ...form, organisation_id: event.target.value || null })}
              >
                <MenuItem value="">None</MenuItem>
                {organisations.map((org) => (
                  <MenuItem key={org.id} value={org.id}>{org.name}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEditOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={handleUpdate} disabled={!canSubmit || loading}>
            Save
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
    </>
  );
};
