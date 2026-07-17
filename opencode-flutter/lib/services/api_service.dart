import '../models/auth_response.dart';
import '../models/file_item.dart';
import '../models/session.dart';
import 'dio_client.dart';

class ApiService {
  final DioClient _dioClient;

  ApiService(this._dioClient);

  // ── OpenCode (opencode_poc) APIs ──────────────────────────────────────────

  Future<bool> checkHealth() async {
    try {
      final res = await _dioClient.dio.get('/health');
      if (res.statusCode == null || res.statusCode! >= 300) return false;
      final body = res.data;
      if (body is Map<String, dynamic>) {
        final status = body['status']?.toString() ?? '';
        return status == 'healthy' || status == 'ok';
      }
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<AuthResponse> login(String username, String password) async {
    final res = await _dioClient.dio.post(
      '/api/v1/auth/login',
      data: {'username': username, 'password': password},
    );
    return AuthResponse.fromJson(_extractData(res.data));
  }

  Future<void> logout() async {
    try {
      await _dioClient.dio.post('/api/v1/auth/logout');
    } catch (_) {}
  }

  Future<List<FileItem>> listFiles({int page = 1, int perPage = 20}) async {
    final res = await _dioClient.dio.get('/api/v1/files', queryParameters: {
      'page': page,
      'per_page': perPage,
    });
    final data = _extractData(res.data);
    final items = data['items'] as List<dynamic>? ?? [];
    return items
        .map((item) => FileItem.fromJson(item as Map<String, dynamic>))
        .toList();
  }

  // ── OpenCode Core (V2) APIs ───────────────────────────────────────────────

  Future<bool> checkCoreHealth() async {
    try {
      final res = await _dioClient.coreDio.get('/api/health');
      return (res.statusCode ?? 500) < 300;
    } catch (_) {
      return false;
    }
  }

  Future<Session> createSession({
    String? id,
    String? agent,
    String? model,
  }) async {
    final body = <String, dynamic>{};
    if (id != null) body['id'] = id;
    if (agent != null) body['agent'] = agent;
    if (model != null) body['model'] = {'id': model};

    final res = await _dioClient.coreDio.post('/v2/session', data: body);
    return Session.fromJson(res.data as Map<String, dynamic>);
  }

  Future<void> sendPrompt(String sessionId, String text) async {
    await _dioClient.coreDio.post('/v2/session/$sessionId/prompt', data: {
      'parts': [
        {'type': 'text', 'text': text}
      ],
    });
  }

  Future<List<dynamic>> getMessages(String sessionId, {int limit = 50}) async {
    final res = await _dioClient.coreDio.get(
      '/v2/session/$sessionId/message',
      queryParameters: {'limit': limit},
    );
    final body = res.data as Map<String, dynamic>;
    return body['data'] as List<dynamic>? ?? [];
  }

  // ── Helpers ───────────────────────────────────────────────────────────────

  Map<String, dynamic> _extractData(dynamic body) {
    if (body is Map<String, dynamic>) {
      if (body.containsKey('status') && body['status'] == 'error') {
        throw Exception(body['error']?.toString() ?? 'API error');
      }
      if (body.containsKey('data') && body['data'] is Map<String, dynamic>) {
        return body['data'] as Map<String, dynamic>;
      }
    }
    return body as Map<String, dynamic>? ?? {};
  }
}
