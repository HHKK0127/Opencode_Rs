import '../models/auth_response.dart';
import '../models/file_item.dart';
import '../models/session.dart';
import '../services/api_service.dart';
import 'local_storage_repository.dart';

class ApiRepository {
  final ApiService _apiService;
  final LocalStorageRepository _local;

  ApiRepository({
    required ApiService apiService,
    required LocalStorageRepository local,
  })  : _apiService = apiService,
        _local = local;

  // ── OpenCode (opencode_poc) APIs ──────────────────────────────────────────

  Future<bool> checkHealth() => _apiService.checkHealth();

  Future<AuthResponse> login(String username, String password) async {
    final response = await _apiService.login(username, password);
    await _local.saveToken(response.token);
    await _local.saveUser({'id': response.user.id, 'username': response.user.username});
    return response;
  }

  Future<void> logout() async {
    await _apiService.logout();
    await _local.deleteToken();
  }

  Future<List<FileItem>> listFiles({int page = 1, int perPage = 20}) =>
      _apiService.listFiles(page: page, perPage: perPage);

  // ── OpenCode Core (V2) APIs ───────────────────────────────────────────────

  Future<bool> checkCoreHealth() => _apiService.checkCoreHealth();

  Future<Session> createSession({String? id, String? agent, String? model}) =>
      _apiService.createSession(id: id, agent: agent, model: model);

  Future<void> sendPrompt(String sessionId, String text) =>
      _apiService.sendPrompt(sessionId, text);

  Future<List<dynamic>> getMessages(String sessionId, {int limit = 50}) =>
      _apiService.getMessages(sessionId, limit: limit);
}
