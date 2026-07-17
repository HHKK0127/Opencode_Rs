import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/launcher/launcher_bloc.dart';
import '../blocs/launcher/launcher_event.dart';
import '../blocs/launcher/launcher_state.dart';
import '../widgets/opencode_logo.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final _promptController = TextEditingController();
  final _focusNode = FocusNode();

  @override
  void dispose() {
    _promptController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _submitPrompt() {
    final text = _promptController.text.trim();
    if (text.isEmpty) return;

    final state = context.read<LauncherBloc>().state;
    if (state.currentSession == null) {
      context.read<LauncherBloc>().add(const LauncherSessionCreateRequested());
    } else {
      context.read<LauncherBloc>().add(LauncherPromptSendRequested(
            sessionId: state.currentSession!.id,
            text: text,
          ));
    }
    _promptController.clear();
  }

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<LauncherBloc, LauncherState>(
      builder: (context, state) {
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 680),
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 48),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Spacer(flex: 2),
                // Logo
                _buildLogo(context),
                const SizedBox(height: 56),
                // Prompt input
                _buildPromptInput(context, state),
                const SizedBox(height: 16),
                // Placeholder suggestions
                _buildPlaceholders(context),
                const Spacer(flex: 3),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildLogo(BuildContext context) {
    return Column(
      children: [
        // OpenCode SVG logo
        OpenCodeLogo(
          width: 280,
          height: 50,
          color: Theme.of(context).colorScheme.onSurface,
        ),
        const SizedBox(height: 16),
        Text(
          'The open source AI coding agent',
          style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                color: Theme.of(context).colorScheme.onSurface.withOpacity(0.6),
              ),
        ),
      ],
    );
  }

  Widget _buildPromptInput(BuildContext context, LauncherState state) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
        ),
      ),
      child: Row(
        children: [
          const SizedBox(width: 16),
          Icon(
            Icons.chat_bubble_outline,
            color: Theme.of(context).colorScheme.onSurface.withOpacity(0.5),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: TextField(
              controller: _promptController,
              focusNode: _focusNode,
              decoration: InputDecoration(
                hintText: 'Ask anything about your code...',
                hintStyle: TextStyle(
                  color: Theme.of(context).colorScheme.onSurface.withOpacity(0.4),
                ),
                border: InputBorder.none,
                contentPadding: const EdgeInsets.symmetric(
                  vertical: 16,
                ),
              ),
              maxLines: 1,
              textInputAction: TextInputAction.send,
              onSubmitted: (_) => _submitPrompt(),
            ),
          ),
          Container(
            margin: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primary,
              borderRadius: BorderRadius.circular(8),
            ),
            child: IconButton(
              icon: const Icon(Icons.arrow_upward, size: 20),
              color: Theme.of(context).colorScheme.onPrimary,
              onPressed: state.status == LauncherStatus.loading
                  ? null
                  : _submitPrompt,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPlaceholders(BuildContext context) {
    final placeholders = [
      'Fix a TODO in the codebase',
      'What is the tech stack of this project?',
      'Fix broken tests',
    ];

    return Wrap(
      spacing: 8,
      runSpacing: 8,
      alignment: WrapAlignment.center,
      children: placeholders.map((text) {
        return InkWell(
          onTap: () {
            _promptController.text = text;
            _focusNode.requestFocus();
          },
          borderRadius: BorderRadius.circular(20),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(20),
              border: Border.all(
                color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
              ),
            ),
            child: Text(
              text,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurface.withOpacity(0.7),
                  ),
            ),
          ),
        );
      }).toList(),
    );
  }
}
