import 'dart:io';

import 'package:flutter/material.dart';
import 'package:hive_flutter/hive_flutter.dart';

import 'app.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  try {
    await Hive.initFlutter();
  } catch (e) {
    // Fallback: initialize Hive with a local directory if path_provider fails
    final dir = Directory('${Platform.environment['APPDATA']}\\opencode_flutter');
    if (!dir.existsSync()) {
      dir.createSync(recursive: true);
    }
    Hive.init(dir.path);
  }
  runApp(const OpenCodeApp());
}
