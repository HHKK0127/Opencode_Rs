import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

class OpenCodeLogo extends StatelessWidget {
  final double? width;
  final double? height;
  final Color? color;

  const OpenCodeLogo({
    super.key,
    this.width,
    this.height,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    return SvgPicture.asset(
      'assets/logo-ornate-light.svg',
      width: width,
      height: height,
      colorFilter: color != null
          ? ColorFilter.mode(color!, BlendMode.srcIn)
          : null,
    );
  }
}
