import 'dart:convert';
import 'package:hive_flutter/hive_flutter.dart';
import '../utils/constants.dart';

class LocalStorageRepository {
  Future<void> saveToken(String token) async {
    final box = await Hive.openBox<String>(kHiveBoxAuth);
    await box.put('token', token);
  }

  Future<String?> getToken() async {
    final box = await Hive.openBox<String>(kHiveBoxAuth);
    return box.get('token');
  }

  Future<void> deleteToken() async {
    final box = await Hive.openBox<String>(kHiveBoxAuth);
    await box.delete('token');
  }

  Future<void> saveUser(Map<String, dynamic> user) async {
    final box = await Hive.openBox<String>(kHiveBoxAuth);
    await box.put('user', jsonEncode(user));
  }

  Future<Map<String, dynamic>?> getUser() async {
    final box = await Hive.openBox<String>(kHiveBoxAuth);
    final raw = box.get('user');
    if (raw == null) return null;
    return jsonDecode(raw) as Map<String, dynamic>;
  }

  Future<void> saveSetting(String key, String value) async {
    final box = await Hive.openBox<String>(kHiveBoxSettings);
    await box.put(key, value);
  }

  Future<String?> getSetting(String key) async {
    final box = await Hive.openBox<String>(kHiveBoxSettings);
    return box.get(key);
  }

  Future<void> clearAll() async {
    await Hive.deleteBoxFromDisk(kHiveBoxAuth);
    await Hive.deleteBoxFromDisk(kHiveBoxSettings);
    await Hive.deleteBoxFromDisk(kHiveBoxCache);
  }
}
