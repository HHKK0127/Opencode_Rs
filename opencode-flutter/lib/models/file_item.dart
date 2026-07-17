import 'package:json_annotation/json_annotation.dart';

part 'file_item.g.dart';

@JsonSerializable()
class FileItem {
  final String id;
  final String name;
  final String path;
  final int size;
  final String? mimeType;
  final DateTime? createdAt;
  final DateTime? updatedAt;

  const FileItem({
    required this.id,
    required this.name,
    required this.path,
    required this.size,
    this.mimeType,
    this.createdAt,
    this.updatedAt,
  });

  factory FileItem.fromJson(Map<String, dynamic> json) =>
      _$FileItemFromJson(json);
  Map<String, dynamic> toJson() => _$FileItemToJson(this);
}
