import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '5m',
  thresholds: {
    'http_req_duration': ['p(95)<500'],
  },
};

export default function () {
  const base_url = __ENV.BASE_URL || 'http://localhost:8080';
  
  // 1. ログイン
  const login_res = http.post(`${base_url}/api/v1/auth/login`, {
    username: 'testuser',
    password: 'testpassword',
  });
  
  check(login_res, {
    'login ok': (r) => r.status === 200,
  });
  
  const token = login_res.json('token');
  
  // 2. ファイルリスト取得
  const list_res = http.get(`${base_url}/api/v1/files`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  
  check(list_res, {
    'list ok': (r) => r.status === 200,
  });
  
  // 3. セッション情報取得
  const session_res = http.get(`${base_url}/api/v1/sessions/info`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  
  check(session_res, {
    'session ok': (r) => r.status === 200,
  });
  
  sleep(1);
}
