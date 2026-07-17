import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/auth/auth_bloc.dart';
import '../blocs/auth/auth_event.dart';
import '../blocs/launcher/launcher_bloc.dart';
import '../blocs/launcher/launcher_event.dart';
import '../blocs/launcher/launcher_state.dart';
import '../widgets/opencode_logo.dart';
import 'ai_terminal_screen.dart';
import 'file_browser_screen.dart';
import 'home_screen.dart';
import 'settings_screen.dart';

class LauncherScreen extends StatefulWidget {
  const LauncherScreen({super.key});

  @override
  State<LauncherScreen> createState() => _LauncherScreenState();
}

class _LauncherScreenState extends State<LauncherScreen> {
  int _currentIndex = 0;
  bool _isRailExpanded = true;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          // Navigation Rail (sidebar)
          _buildNavigationRail(context),
          // Main content
          Expanded(
            child: _buildMainContent(),
          ),
        ],
      ),
    );
  }

  Widget _buildNavigationRail(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          right: BorderSide(
            color: Theme.of(context).colorScheme.outline.withOpacity(0.2),
          ),
        ),
      ),
      child: NavigationRail(
        selectedIndex: _currentIndex,
        onDestinationSelected: (i) => setState(() => _currentIndex = i),
        extended: _isRailExpanded,
        backgroundColor: Colors.transparent,
        leading: Column(
          children: [
            const SizedBox(height: 12),
            // OpenCode logo
            if (_isRailExpanded)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: OpenCodeLogo(
                  width: 140,
                  height: 25,
                  color: Theme.of(context).colorScheme.onSurface,
                ),
              )
            else
              Icon(
                Icons.code_rounded,
                color: Theme.of(context).colorScheme.onSurface,
                size: 24,
              ),
            const SizedBox(height: 8),
            IconButton(
              icon: Icon(
                _isRailExpanded ? Icons.chevron_left : Icons.chevron_right,
                size: 20,
              ),
              onPressed: () => setState(() => _isRailExpanded = !_isRailExpanded),
            ),
            const SizedBox(height: 8),
          ],
        ),
        trailing: Expanded(
          child: Align(
            alignment: Alignment.bottomCenter,
            child: Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: IconButton(
                icon: const Icon(Icons.logout, size: 20),
                onPressed: () {
                  context.read<AuthBloc>().add(const AuthLogoutRequested());
                },
              ),
            ),
          ),
        ),
        destinations: const [
          NavigationRailDestination(
            icon: Icon(Icons.home_outlined, size: 20),
            selectedIcon: Icon(Icons.home, size: 20),
            label: Text('Home'),
          ),
          NavigationRailDestination(
            icon: Icon(Icons.terminal_outlined, size: 20),
            selectedIcon: Icon(Icons.terminal, size: 20),
            label: Text('AI Terminal'),
          ),
          NavigationRailDestination(
            icon: Icon(Icons.folder_outlined, size: 20),
            selectedIcon: Icon(Icons.folder, size: 20),
            label: Text('Files'),
          ),
          NavigationRailDestination(
            icon: Icon(Icons.settings_outlined, size: 20),
            selectedIcon: Icon(Icons.settings, size: 20),
            label: Text('Settings'),
          ),
        ],
      ),
    );
  }

  Widget _buildMainContent() {
    return IndexedStack(
      index: _currentIndex,
      children: const [
        HomeScreen(),
        AiTerminalScreen(),
        FileBrowserScreen(),
        SettingsScreen(),
      ],
    );
  }
}

class _LauncherHome extends StatelessWidget {
  const _LauncherHome();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<LauncherBloc, LauncherState>(
      builder: (context, state) {
        return Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildModeSelector(context, state),
              const SizedBox(height: 16),
              _buildStatusIndicator(context, state),
              const SizedBox(height: 16),
              _buildControlButtons(context, state),
              const SizedBox(height: 24),
              _buildHealthInfo(context, state),
              const Spacer(),
              if (state.errorMessage != null)
                _buildErrorBanner(context, state),
            ],
          ),
        );
      },
    );
  }

  Widget _buildModeSelector(BuildContext context, LauncherState state) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Execution Mode',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            SegmentedButton<LauncherMode>(
              segments: const [
                ButtonSegment(
                  value: LauncherMode.opencode,
                  label: Text('OpenCode'),
                  icon: Icon(Icons.code),
                ),
                ButtonSegment(
                  value: LauncherMode.aiTerminal,
                  label: Text('AI Terminal'),
                  icon: Icon(Icons.terminal),
                ),
              ],
              selected: {state.mode},
              onSelectionChanged: (modes) {
                context.read<LauncherBloc>().add(
                      LauncherModeChanged(mode: modes.first),
                    );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusIndicator(BuildContext context, LauncherState state) {
    final statusText = switch (state.status) {
      LauncherStatus.initial => 'Ready',
      LauncherStatus.loading => 'Loading...',
      LauncherStatus.running => 'Running',
      LauncherStatus.stopped => 'Stopped',
      LauncherStatus.error => 'Error',
    };

    final statusColor = switch (state.status) {
      LauncherStatus.initial => Colors.grey,
      LauncherStatus.loading => Colors.orange,
      LauncherStatus.running => Colors.green,
      LauncherStatus.stopped => Colors.red,
      LauncherStatus.error => Colors.red,
    };

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(Icons.circle, color: statusColor, size: 16),
            const SizedBox(width: 8),
            Text(
              'Status: $statusText',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const Spacer(),
            if (state.lastHealthCheck != null)
              Text(
                'Last check: ${state.lastHealthCheck!.hour}:${state.lastHealthCheck!.minute.toString().padLeft(2, '0')}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildControlButtons(BuildContext context, LauncherState state) {
    final isLoading = state.status == LauncherStatus.loading;

    return Row(
      children: [
        Expanded(
          child: FilledButton.icon(
            onPressed: isLoading
                ? null
                : () => context
                    .read<LauncherBloc>()
                    .add(const LauncherStartRequested()),
            icon: const Icon(Icons.play_arrow),
            label: const Text('Start'),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: FilledButton.icon(
            onPressed: isLoading
                ? null
                : () => context
                    .read<LauncherBloc>()
                    .add(const LauncherStopRequested()),
            icon: const Icon(Icons.stop),
            label: const Text('Stop'),
            style: FilledButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.error,
            ),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: OutlinedButton.icon(
            onPressed: isLoading
                ? null
                : () => context
                    .read<LauncherBloc>()
                    .add(const LauncherReloadRequested()),
            icon: const Icon(Icons.refresh),
            label: const Text('Reload'),
          ),
        ),
      ],
    );
  }

  Widget _buildHealthInfo(BuildContext context, LauncherState state) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Backend Health',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            _buildHealthRow('OpenCode API', state.opencodeHealthy),
            const SizedBox(height: 4),
            _buildHealthRow('Core API', state.coreHealthy),
          ],
        ),
      ),
    );
  }

  Widget _buildHealthRow(String label, bool healthy) {
    return Row(
      children: [
        Icon(
          healthy ? Icons.check_circle : Icons.error,
          color: healthy ? Colors.green : Colors.red,
          size: 16,
        ),
        const SizedBox(width: 8),
        Text(label),
      ],
    );
  }

  Widget _buildErrorBanner(BuildContext context, LauncherState state) {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(
              Icons.error_outline,
              color: Theme.of(context).colorScheme.error,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                state.errorMessage!,
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onErrorContainer,
                ),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: () {
                // Clear error by reloading
                context
                    .read<LauncherBloc>()
                    .add(const LauncherReloadRequested());
              },
            ),
          ],
        ),
      ),
    );
  }
}
