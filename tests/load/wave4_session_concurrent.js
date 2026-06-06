import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '5m', target: 5000 },
    { duration: '10m', target: 5000 },
    { duration: '5m', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<100'],
  },
};

export default function () {
  const base_url = __ENV.BASE_URL || 'http://localhost:8080';
  
  const login_res = http.post(`${base_url}/api/v1/auth/login`, {
    username: 'testuser',
    password: 'testpassword',
  });
  
  const token = login_res.json('token');
  
  // セッション検証
  const session_res = http.post(`${base_url}/api/v1/sessions/validate`, {}, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  
  check(session_res, {
    'session valid': (r) => r.status === 200,
  });
  
  sleep(0.5);
}
