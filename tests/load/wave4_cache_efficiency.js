import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '5m', target: 100 },
    { duration: '15m', target: 100 },
    { duration: '5m', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<1000'],
  },
};

export default function () {
  const base_url = __ENV.BASE_URL || 'http://localhost:8080';
  
  // ログイン
  const login_res = http.post(`${base_url}/api/v1/auth/login`, {
    username: 'testuser',
    password: 'testpassword',
  });
  
  check(login_res, {
    'login status': (r) => r.status === 200,
  });
  
  const token = login_res.json('token');
  
  // ファイルリスト取得（キャッシュテスト）
  for (let i = 0; i < 10; i++) {
    const list_res = http.get(`${base_url}/api/v1/files?page=1&per_page=20`, {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    
    check(list_res, {
      'list status': (r) => r.status === 200,
    });
  }
  
  sleep(1);
}
