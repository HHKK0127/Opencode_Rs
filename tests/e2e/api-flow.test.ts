import { describe, it, expect, beforeAll, afterAll } from 'vitest';

const BASE_URL = 'http://localhost:8080/api/v1';
let authToken: string;
let username: string;

describe('E2E API Flow', () => {
  beforeAll(async () => {
    // Server should be running before tests start
    const healthRes = await fetch(`${BASE_URL}/health`);
    expect(healthRes.status).toBe(200);
  });

  describe('Authentication Flow', () => {
    it('should register a new user', async () => {
      username = `e2euser_${Date.now()}`;

      const res = await fetch(`${BASE_URL}/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username,
          password: 'TestPass123!',
        }),
      });

      expect(res.status).toBe(200);
      const data = await res.json() as Record<string, unknown>;
      expect(data.token).toBeDefined();
      expect(typeof data.token).toBe('string');
    });

    it('should login user and get token', async () => {
      const res = await fetch(`${BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username,
          password: 'TestPass123!',
        }),
      });

      expect(res.status).toBe(200);
      const data = await res.json() as Record<string, unknown>;
      expect(data.token).toBeDefined();
      authToken = data.token as string;
    });

    it('should refresh token', async () => {
      const res = await fetch(`${BASE_URL}/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${authToken}`,
        },
      });

      expect(res.status).toBe(200);
      const data = await res.json() as Record<string, unknown>;
      expect(data.token).toBeDefined();
    });

    it('should reject login with wrong password', async () => {
      const res = await fetch(`${BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username,
          password: 'WrongPassword',
        }),
      });

      expect(res.status).toBe(401);
    });
  });

  describe('Protected Endpoints', () => {
    it('should access protected endpoint with valid token', async () => {
      const res = await fetch(`${BASE_URL}/users`, {
        method: 'GET',
        headers: { 'Authorization': `Bearer ${authToken}` },
      });

      expect(res.status).toBe(200);
    });

    it('should reject protected endpoint without token', async () => {
      const res = await fetch(`${BASE_URL}/users`, {
        method: 'GET',
      });

      expect(res.status).toBe(401);
    });

    it('should reject protected endpoint with invalid token', async () => {
      const res = await fetch(`${BASE_URL}/users`, {
        method: 'GET',
        headers: { 'Authorization': 'Bearer invalid_token' },
      });

      expect(res.status).toBe(401);
    });
  });

  describe('Health Check', () => {
    it('should return healthy status', async () => {
      const res = await fetch(`${BASE_URL}/health`);

      expect(res.status).toBe(200);
      const data = await res.json() as Record<string, unknown>;
      expect(data.status).toBe('healthy');
    });

    it('should verify database health', async () => {
      const res = await fetch(`${BASE_URL}/health/db`);

      expect(res.status).toBe(200);
      const data = await res.json() as Record<string, unknown>;
      expect(data.status).toBe('healthy');
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid JSON gracefully', async () => {
      const res = await fetch(`${BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: 'invalid json',
      });

      expect(res.status).toBe(400);
    });

    it('should handle missing required fields', async () => {
      const res = await fetch(`${BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      });

      expect(res.status).toBe(400);
    });
  });

  afterAll(async () => {
    // Cleanup if needed
    console.log('E2E tests completed');
  });
});
