import 'package:equatable/equatable.dart';
import '../../models/session.dart';
import 'launcher_event.dart';

enum LauncherStatus { initial, loading, running, stopped, error }

class LauncherState extends Equatable {
  final LauncherMode mode;
  final LauncherStatus status;
  final bool opencodeHealthy;
  final bool coreHealthy;
  final Session? currentSession;
  final List<dynamic> messages;
  final String? errorMessage;
  final DateTime? lastHealthCheck;

  const LauncherState({
    this.mode = LauncherMode.opencode,
    this.status = LauncherStatus.initial,
    this.opencodeHealthy = false,
    this.coreHealthy = false,
    this.currentSession,
    this.messages = const [],
    this.errorMessage,
    this.lastHealthCheck,
  });

  LauncherState copyWith({
    LauncherMode? mode,
    LauncherStatus? status,
    bool? opencodeHealthy,
    bool? coreHealthy,
    Session? currentSession,
    List<dynamic>? messages,
    String? errorMessage,
    DateTime? lastHealthCheck,
  }) {
    return LauncherState(
      mode: mode ?? this.mode,
      status: status ?? this.status,
      opencodeHealthy: opencodeHealthy ?? this.opencodeHealthy,
      coreHealthy: coreHealthy ?? this.coreHealthy,
      currentSession: currentSession ?? this.currentSession,
      messages: messages ?? this.messages,
      errorMessage: errorMessage ?? this.errorMessage,
      lastHealthCheck: lastHealthCheck ?? this.lastHealthCheck,
    );
  }

  @override
  List<Object?> get props => [
        mode,
        status,
        opencodeHealthy,
        coreHealthy,
        currentSession,
        messages,
        errorMessage,
        lastHealthCheck,
      ];
}
