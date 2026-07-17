import 'package:dio/dio.dart';
import '../utils/constants.dart';

class DioClient {
  late final Dio _dio;
  late final Dio _coreDio;

  DioClient() {
    _dio = _createDio(kApiBaseUrl);
    _coreDio = _createDio(kCoreApiBaseUrl);
  }

  Dio _createDio(String baseUrl) {
    final dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: kConnectTimeout,
      receiveTimeout: kReceiveTimeout,
      headers: {'Content-Type': 'application/json'},
    ));

    dio.interceptors.add(LogInterceptor(
      requestBody: true,
      responseBody: true,
      error: true,
    ));

    return dio;
  }

  Dio get dio => _dio;
  Dio get coreDio => _coreDio;

  void setAuthToken(String token) {
    final authHeader = {'Authorization': 'Bearer $token'};
    _dio.options.headers.addAll(authHeader);
    _coreDio.options.headers.addAll(authHeader);
  }

  void clearAuthToken() {
    _dio.options.headers.remove('Authorization');
    _coreDio.options.headers.remove('Authorization');
  }
}
