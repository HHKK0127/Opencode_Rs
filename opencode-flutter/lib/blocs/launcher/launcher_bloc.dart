import 'package:flutter_bloc/flutter_bloc.dart';
import '../../repositories/api_repository.dart';
import 'launcher_event.dart';
import 'launcher_state.dart';

class LauncherBloc extends Bloc<LauncherEvent, LauncherState> {
  final ApiRepository _apiRepository;

  LauncherBloc({required ApiRepository apiRepository})
      : _apiRepository = apiRepository,
        super(const LauncherState()) {
    on<LauncherModeChanged>(_onModeChanged);
    on<LauncherStartRequested>(_onStartRequested);
    on<LauncherStopRequested>(_onStopRequested);
    on<LauncherReloadRequested>(_onReloadRequested);
    on<LauncherHealthCheckRequested>(_onHealthCheckRequested);
    on<LauncherSessionCreateRequested>(_onSessionCreateRequested);
    on<LauncherPromptSendRequested>(_onPromptSendRequested);
  }

  void _onModeChanged(
    LauncherModeChanged event,
    Emitter<LauncherState> emit,
  ) {
    emit(state.copyWith(mode: event.mode));
  }

  Future<void> _onStartRequested(
    LauncherStartRequested event,
    Emitter<LauncherState> emit,
  ) async {
    emit(state.copyWith(status: LauncherStatus.loading));
    try {
      final opencodeOk = await _apiRepository.checkHealth();
      final coreOk = await _apiRepository.checkCoreHealth();

      emit(state.copyWith(
        opencodeHealthy: opencodeOk,
        coreHealthy: coreOk,
        status: LauncherStatus.running,
        lastHealthCheck: DateTime.now(),
      ));
    } catch (e) {
      emit(state.copyWith(
        status: LauncherStatus.error,
        errorMessage: e.toString(),
      ));
    }
  }

  Future<void> _onStopRequested(
    LauncherStopRequested event,
    Emitter<LauncherState> emit,
  ) async {
    emit(state.copyWith(
      status: LauncherStatus.stopped,
      currentSession: null,
      messages: [],
    ));
  }

  Future<void> _onReloadRequested(
    LauncherReloadRequested event,
    Emitter<LauncherState> emit,
  ) async {
    add(const LauncherHealthCheckRequested());
  }

  Future<void> _onHealthCheckRequested(
    LauncherHealthCheckRequested event,
    Emitter<LauncherState> emit,
  ) async {
    try {
      final opencodeOk = await _apiRepository.checkHealth();
      final coreOk = await _apiRepository.checkCoreHealth();

      emit(state.copyWith(
        opencodeHealthy: opencodeOk,
        coreHealthy: coreOk,
        lastHealthCheck: DateTime.now(),
      ));
    } catch (e) {
      emit(state.copyWith(
        errorMessage: e.toString(),
      ));
    }
  }

  Future<void> _onSessionCreateRequested(
    LauncherSessionCreateRequested event,
    Emitter<LauncherState> emit,
  ) async {
    emit(state.copyWith(status: LauncherStatus.loading));
    try {
      final session = await _apiRepository.createSession(
        agent: event.agent,
        model: event.model,
      );
      emit(state.copyWith(
        currentSession: session,
        status: LauncherStatus.running,
      ));
    } catch (e) {
      emit(state.copyWith(
        status: LauncherStatus.error,
        errorMessage: e.toString(),
      ));
    }
  }

  Future<void> _onPromptSendRequested(
    LauncherPromptSendRequested event,
    Emitter<LauncherState> emit,
  ) async {
    try {
      await _apiRepository.sendPrompt(event.sessionId, event.text);
      final messages = await _apiRepository.getMessages(event.sessionId);
      emit(state.copyWith(messages: messages));
    } catch (e) {
      emit(state.copyWith(
        errorMessage: e.toString(),
      ));
    }
  }
}
