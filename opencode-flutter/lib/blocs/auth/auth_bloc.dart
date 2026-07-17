import 'package:flutter_bloc/flutter_bloc.dart';
import '../../models/user.dart';
import '../../repositories/auth_repository.dart';
import '../../utils/constants.dart';
import 'auth_event.dart';
import 'auth_state.dart';

class AuthBloc extends Bloc<AuthEvent, AuthState> {
  final AuthRepository _authRepository;

  AuthBloc({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const AuthInitial()) {
    on<AuthCheckRequested>(_onAuthCheckRequested);
    on<AuthLoginRequested>(_onAuthLoginRequested);
    on<AuthLogoutRequested>(_onAuthLogoutRequested);
    on<AuthTestBypassRequested>(_onAuthTestBypassRequested);
  }

  Future<void> _onAuthCheckRequested(
    AuthCheckRequested event,
    Emitter<AuthState> emit,
  ) async {
    emit(const AuthLoading());
    try {
      final isAuth = await _authRepository.isAuthenticated();
      if (isAuth) {
        final token = await _authRepository.getToken();
        final userMap = await _authRepository.getUser();
        if (token != null && userMap != null) {
          final user = User.fromJson(userMap);
          emit(AuthAuthenticated(token: token, user: user));
          return;
        }
      }
      emit(const AuthUnauthenticated());
    } catch (e) {
      emit(AuthFailure(message: e.toString()));
    }
  }

  Future<void> _onAuthLoginRequested(
    AuthLoginRequested event,
    Emitter<AuthState> emit,
  ) async {
    emit(const AuthLoading());
    try {
      final result = await _authRepository.login(event.username, event.password);
      final userMap = await _authRepository.getUser();
      final user = userMap != null ? User.fromJson(userMap) : User(id: 'temp', username: result.username);
      emit(AuthAuthenticated(token: result.token, user: user));
    } catch (e) {
      emit(AuthFailure(message: e.toString()));
    }
  }

  Future<void> _onAuthLogoutRequested(
    AuthLogoutRequested event,
    Emitter<AuthState> emit,
  ) async {
    emit(const AuthLoading());
    try {
      await _authRepository.logout();
      emit(const AuthUnauthenticated());
    } catch (e) {
      emit(AuthFailure(message: e.toString()));
    }
  }

  Future<void> _onAuthTestBypassRequested(
    AuthTestBypassRequested event,
    Emitter<AuthState> emit,
  ) async {
    if (!kAllowTestLoginBypass) {
      emit(const AuthFailure(message: 'Test bypass is not allowed'));
      return;
    }
    emit(const AuthAuthenticated(
      token: 'test-token',
      user: User(id: 'test-user', username: 'Test User'),
    ));
  }
}
