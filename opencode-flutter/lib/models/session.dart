import 'package:json_annotation/json_annotation.dart';

part 'session.g.dart';

@JsonSerializable()
class Session {
  final String id;
  final String? agent;
  final String? model;
  final DateTime? createdAt;
  final DateTime? updatedAt;
  final String? status;

  const Session({
    required this.id,
    this.agent,
    this.model,
    this.createdAt,
    this.updatedAt,
    this.status,
  });

  factory Session.fromJson(Map<String, dynamic> json) =>
      _$SessionFromJson(json);
  Map<String, dynamic> toJson() => _$SessionToJson(this);
}
