import 'package:equatable/equatable.dart';

enum LauncherMode { opencode, aiTerminal }

abstract class LauncherEvent extends Equatable {
  const LauncherEvent();

  @override
  List<Object?> get props => [];
}

class LauncherModeChanged extends LauncherEvent {
  final LauncherMode mode;

  const LauncherModeChanged({required this.mode});

  @override
  List<Object?> get props => [mode];
}

class LauncherStartRequested extends LauncherEvent {
  const LauncherStartRequested();
}

class LauncherStopRequested extends LauncherEvent {
  const LauncherStopRequested();
}

class LauncherReloadRequested extends LauncherEvent {
  const LauncherReloadRequested();
}

class LauncherHealthCheckRequested extends LauncherEvent {
  const LauncherHealthCheckRequested();
}

class LauncherSessionCreateRequested extends LauncherEvent {
  final String? agent;
  final String? model;

  const LauncherSessionCreateRequested({this.agent, this.model});

  @override
  List<Object?> get props => [agent, model];
}

class LauncherPromptSendRequested extends LauncherEvent {
  final String sessionId;
  final String text;

  const LauncherPromptSendRequested({
    required this.sessionId,
    required this.text,
  });

  @override
  List<Object?> get props => [sessionId, text];
}
