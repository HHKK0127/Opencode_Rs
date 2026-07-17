const String kApiBaseUrl = String.fromEnvironment(
  'API_BASE_URL',
  defaultValue: 'http://localhost:8080',
);

const String kCoreApiBaseUrl = String.fromEnvironment(
  'CORE_API_BASE_URL',
  defaultValue: 'http://localhost:4096',
);

const bool kAllowTestLoginBypass = bool.fromEnvironment(
  'ALLOW_TEST_LOGIN_BYPASS',
  defaultValue: true,
);

const Duration kConnectTimeout = Duration(seconds: 10);
const Duration kReceiveTimeout = Duration(seconds: 30);

const String kHiveBoxAuth = 'auth';
const String kHiveBoxSettings = 'settings';
const String kHiveBoxCache = 'cache';
