import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../blocs/launcher/launcher_bloc.dart';
import '../blocs/launcher/launcher_event.dart';
import '../blocs/launcher/launcher_state.dart';
import '../repositories/local_storage_repository.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  final _apiUrlController = TextEditingController();
  final _coreUrlController = TextEditingController();
  final _workPathController = TextEditingController();
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final local = context.read<LocalStorageRepository>();
    final apiUrl = await local.getSetting('api_url') ?? 'http://localhost:8080';
    final coreUrl = await local.getSetting('core_url') ?? 'http://localhost:4096';
    final workPath = await local.getSetting('work_path') ?? '';

    setState(() {
      _apiUrlController.text = apiUrl;
      _coreUrlController.text = coreUrl;
      _workPathController.text = workPath;
      _loading = false;
    });
  }

  Future<void> _saveSettings() async {
    final local = context.read<LocalStorageRepository>();
    await local.saveSetting('api_url', _apiUrlController.text.trim());
    await local.saveSetting('core_url', _coreUrlController.text.trim());
    await local.saveSetting('work_path', _workPathController.text.trim());

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Settings saved. Restart to apply.')),
      );
    }
  }

  @override
  void dispose() {
    _apiUrlController.dispose();
    _coreUrlController.dispose();
    _workPathController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    return Padding(
      padding: const EdgeInsets.all(24),
      child: ListView(
        children: [
          Text(
            'Settings',
            style: Theme.of(context).textTheme.headlineMedium,
          ),
          const SizedBox(height: 24),
          _buildSection(
            context,
            title: 'API Configuration',
            icon: Icons.api,
            children: [
              _buildTextField(
                controller: _apiUrlController,
                label: 'OpenCode API URL',
                hint: 'http://localhost:8080',
                icon: Icons.cloud,
              ),
              const SizedBox(height: 16),
              _buildTextField(
                controller: _coreUrlController,
                label: 'Core API URL',
                hint: 'http://localhost:4096',
                icon: Icons.dns,
              ),
            ],
          ),
          const SizedBox(height: 24),
          _buildSection(
            context,
            title: 'Workspace',
            icon: Icons.folder,
            children: [
              _buildTextField(
                controller: _workPathController,
                label: 'Working Directory',
                hint: 'Path to your project',
                icon: Icons.drive_file_move,
              ),
            ],
          ),
          const SizedBox(height: 24),
          _buildSection(
            context,
            title: 'Default Mode',
            icon: Icons.toggle_on,
            children: [
              BlocBuilder<LauncherBloc, LauncherState>(
                builder: (context, state) {
                  return SegmentedButton<LauncherMode>(
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
                  );
                },
              ),
            ],
          ),
          const SizedBox(height: 32),
          FilledButton.icon(
            onPressed: _saveSettings,
            icon: const Icon(Icons.save),
            label: const Text('Save Settings'),
          ),
        ],
      ),
    );
  }

  Widget _buildSection(
    BuildContext context, {
    required String title,
    required IconData icon,
    required List<Widget> children,
  }) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, size: 20),
                const SizedBox(width: 8),
                Text(
                  title,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 16),
            ...children,
          ],
        ),
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    required String label,
    required String hint,
    required IconData icon,
  }) {
    return TextField(
      controller: controller,
      decoration: InputDecoration(
        labelText: label,
        hintText: hint,
        prefixIcon: Icon(icon),
        border: const OutlineInputBorder(),
      ),
    );
  }
}
