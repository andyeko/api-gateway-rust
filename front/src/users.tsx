import { useEffect, useMemo, useState } from "react";
import {
  Alert,
  Box,
  Button,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  Paper,
  Stack,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  TextField,
  Typography
} from "@mui/material";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";

type User = {
  id: string;
  email: string;
  name: string;
  created_at: string;
};

type UserInput = {
  email: string;
  name: string;
};

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:4001";

export const UsersPage = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [form, setForm] = useState<UserInput>({ email: "", name: "" });
  const [editId, setEditId] = useState<string | null>(null);

  const canSubmit = useMemo(() => form.email.trim() && form.name.trim(), [form]);

  const loadUsers = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(
        `${API_BASE}/users?_start=0&_end=100`,
        { headers: { Accept: "application/json" } }
      );
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

  useEffect(() => {
    loadUsers();
  }, []);

  const resetForm = () => setForm({ email: "", name: "" });

  const openCreate = () => {
    resetForm();
    setCreateOpen(true);
  };

  const openEdit = (user: User) => {
    setForm({ email: user.email, name: user.name });
    setEditId(user.id);
    setEditOpen(true);
  };

  const handleCreate = async () => {
    if (!canSubmit) return;
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE}/users`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form)
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
      const response = await fetch(`${API_BASE}/users/${editId}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form)
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
      const response = await fetch(`${API_BASE}/users/${id}`, { method: "DELETE" });
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

  return (
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
                <TableCell>ID</TableCell>
                <TableCell>Email</TableCell>
                <TableCell>Name</TableCell>
                <TableCell>Created</TableCell>
                <TableCell align="right">Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {users.map((user) => (
                <TableRow key={user.id} hover>
                  <TableCell>{user.id}</TableCell>
                  <TableCell>{user.email}</TableCell>
                  <TableCell>{user.name}</TableCell>
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
                  <TableCell colSpan={5} align="center">
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
  );
};
