import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/launcher/launcher_bloc.dart';
import '../blocs/launcher/launcher_event.dart';
import '../blocs/launcher/launcher_state.dart';

class AiTerminalScreen extends StatefulWidget {
  const AiTerminalScreen({super.key});

  @override
  State<AiTerminalScreen> createState() => _AiTerminalScreenState();
}

class _AiTerminalScreenState extends State<AiTerminalScreen> {
  final _promptController = TextEditingController();
  final _scrollController = ScrollController();

  @override
  void dispose() {
    _promptController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _sendPrompt() {
    final text = _promptController.text.trim();
    if (text.isEmpty) return;

    final state = context.read<LauncherBloc>().state;
    if (state.currentSession == null) {
      // Create session first
      context.read<LauncherBloc>().add(const LauncherSessionCreateRequested());
      // TODO: Queue prompt to send after session creation
      return;
    }

    context.read<LauncherBloc>().add(LauncherPromptSendRequested(
          sessionId: state.currentSession!.id,
          text: text,
        ));
    _promptController.clear();
  }

  void _createSession() {
    context.read<LauncherBloc>().add(const LauncherSessionCreateRequested());
  }

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<LauncherBloc, LauncherState>(
      builder: (context, state) {
        return Column(
          children: [
            // Session header
            _buildSessionHeader(context, state),
            // Messages
            Expanded(child: _buildMessageList(context, state)),
            // Prompt input
            _buildPromptInput(context, state),
          ],
        );
      },
    );
  }

  Widget _buildSessionHeader(BuildContext context, LauncherState state) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).dividerColor,
          ),
        ),
      ),
      child: Row(
        children: [
          Icon(
            Icons.terminal,
            color: Theme.of(context).colorScheme.primary,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'AI Terminal',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (state.currentSession != null)
                  Text(
                    'Session: ${state.currentSession!.id}',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
              ],
            ),
          ),
          if (state.status == LauncherStatus.loading)
            const SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
          const SizedBox(width: 8),
          FilledButton.tonalIcon(
            onPressed: state.status == LauncherStatus.loading
                ? null
                : _createSession,
            icon: const Icon(Icons.add, size: 18),
            label: const Text('New Session'),
          ),
        ],
      ),
    );
  }

  Widget _buildMessageList(BuildContext context, LauncherState state) {
    if (state.messages.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.chat_bubble_outline,
              size: 64,
              color: Theme.of(context).colorScheme.onSurfaceVariant.withOpacity(0.5),
            ),
            const SizedBox(height: 16),
            Text(
              'No messages yet',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
            const SizedBox(height: 8),
            Text(
              'Create a session and send a prompt to get started',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: state.messages.length,
      itemBuilder: (context, index) {
        final message = state.messages[index] as Map<String, dynamic>;
        final role = message['role'] as String? ?? 'unknown';
        final parts = message['parts'] as List<dynamic>? ?? [];
        final text = parts.isNotEmpty
            ? (parts[0] as Map<String, dynamic>)['text'] as String? ?? ''
            : '';
        final isUser = role == 'user';

        return Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Avatar
              CircleAvatar(
                radius: 16,
                backgroundColor: isUser
                    ? Theme.of(context).colorScheme.primary
                    : Theme.of(context).colorScheme.secondary,
                child: Icon(
                  isUser ? Icons.person : Icons.smart_toy,
                  size: 18,
                  color: isUser
                      ? Theme.of(context).colorScheme.onPrimary
                      : Theme.of(context).colorScheme.onSecondary,
                ),
              ),
              const SizedBox(width: 12),
              // Message content
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      isUser ? 'You' : 'Assistant',
                      style: Theme.of(context).textTheme.labelMedium?.copyWith(
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                    const SizedBox(height: 4),
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: isUser
                            ? Theme.of(context)
                                .colorScheme
                                .primaryContainer
                                .withOpacity(0.3)
                            : Theme.of(context)
                                .colorScheme
                                .surfaceContainerHighest,
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(text),
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildPromptInput(BuildContext context, LauncherState state) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          top: BorderSide(
            color: Theme.of(context).dividerColor,
          ),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _promptController,
              decoration: InputDecoration(
                hintText: 'Enter your prompt...',
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(24),
                ),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 20,
                  vertical: 12,
                ),
                prefixIcon: const Icon(Icons.chat_bubble_outline),
              ),
              maxLines: null,
              textInputAction: TextInputAction.send,
              onSubmitted: (_) => _sendPrompt(),
            ),
          ),
          const SizedBox(width: 12),
          IconButton(
            onPressed: state.status == LauncherStatus.loading
                ? null
                : _sendPrompt,
            icon: const Icon(Icons.send),
            style: IconButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.primary,
              foregroundColor: Theme.of(context).colorScheme.onPrimary,
              padding: const EdgeInsets.all(12),
            ),
          ),
        ],
      ),
    );
  }
}
