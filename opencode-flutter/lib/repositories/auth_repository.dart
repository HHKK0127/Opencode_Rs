import '../services/api_service.dart';
import 'local_storage_repository.dart';

class AuthRepository {
  final ApiService _apiService;
  final LocalStorageRepository _local;

  AuthRepository({
    required ApiService apiService,
    required LocalStorageRepository local,
  })  : _apiService = apiService,
        _local = local;

  Future<({String token, String username})> login(
    String username,
    String password,
  ) async {
    final response = await _apiService.login(username, password);
    await _local.saveToken(response.token);
    await _local.saveUser({'id': response.user.id, 'username': response.user.username});
    return (token: response.token, username: response.user.username);
  }

  Future<void> logout() async {
    await _apiService.logout();
    await _local.deleteToken();
  }

  Future<bool> isAuthenticated() async {
    final t = await _local.getToken();
    return t != null && t.isNotEmpty;
  }

  Future<String?> getToken() => _local.getToken();

  Future<Map<String, dynamic>?> getUser() => _local.getUser();
}
