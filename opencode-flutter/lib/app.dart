import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'blocs/auth/auth_bloc.dart';
import 'blocs/auth/auth_event.dart';
import 'blocs/auth/auth_state.dart';
import 'blocs/launcher/launcher_bloc.dart';
import 'repositories/api_repository.dart';
import 'repositories/auth_repository.dart';
import 'repositories/local_storage_repository.dart';
import 'screens/launcher_screen.dart';
import 'screens/login_screen.dart';
import 'services/api_service.dart';
import 'services/dio_client.dart';

class OpenCodeApp extends StatelessWidget {
  const OpenCodeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider(create: (_) => LocalStorageRepository()),
        RepositoryProvider(create: (_) => DioClient()),
        RepositoryProvider(
          create: (ctx) => ApiService(ctx.read<DioClient>()),
        ),
        RepositoryProvider(
          create: (ctx) => ApiRepository(
            apiService: ctx.read<ApiService>(),
            local: ctx.read<LocalStorageRepository>(),
          ),
        ),
        RepositoryProvider(
          create: (ctx) => AuthRepository(
            apiService: ctx.read<ApiService>(),
            local: ctx.read<LocalStorageRepository>(),
          ),
        ),
      ],
      child: MultiBlocProvider(
        providers: [
          BlocProvider(
            create: (ctx) => AuthBloc(
              authRepository: ctx.read<AuthRepository>(),
            )..add(const AuthCheckRequested()),
          ),
          BlocProvider(
            create: (ctx) => LauncherBloc(
              apiRepository: ctx.read<ApiRepository>(),
            ),
          ),
        ],
        child: MaterialApp(
          title: 'OpenCode',
          theme: ThemeData(
            useMaterial3: true,
            brightness: Brightness.dark,
            colorScheme: ColorScheme.dark(
              primary: const Color(0xFFB7B1B1),
              onPrimary: const Color(0xFF1A1A1A),
              secondary: const Color(0xFF4B4646),
              onSecondary: const Color(0xFFF1ECEC),
              surface: const Color(0xFF1A1A1A),
              onSurface: const Color(0xFFF1ECEC),
              surfaceContainerHighest: const Color(0xFF2A2A2A),
              outline: const Color(0xFF3A3A3A),
            ),
            scaffoldBackgroundColor: const Color(0xFF1A1A1A),
            cardTheme: CardTheme(
              color: const Color(0xFF2A2A2A),
              elevation: 0,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12),
                side: BorderSide(
                  color: const Color(0xFF3A3A3A),
                ),
              ),
            ),
            appBarTheme: const AppBarTheme(
              backgroundColor: Color(0xFF1A1A1A),
              foregroundColor: Color(0xFFF1ECEC),
              elevation: 0,
            ),
            navigationRailTheme: NavigationRailThemeData(
              backgroundColor: const Color(0xFF1A1A1A),
              selectedIconTheme: IconThemeData(
                color: const Color(0xFFF1ECEC),
              ),
              unselectedIconTheme: IconThemeData(
                color: const Color(0xFF6A6A6A),
              ),
              selectedLabelTextStyle: TextStyle(
                color: const Color(0xFFF1ECEC),
              ),
              unselectedLabelTextStyle: TextStyle(
                color: const Color(0xFF6A6A6A),
              ),
            ),
          ),
          home: const AuthGate(),
        ),
      ),
    );
  }
}

class AuthGate extends StatelessWidget {
  const AuthGate({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AuthBloc, AuthState>(
      builder: (context, state) {
        if (state is AuthAuthenticated) {
          return const LauncherScreen();
        }
        if (state is AuthLoading) {
          return const Scaffold(
            body: Center(child: CircularProgressIndicator()),
          );
        }
        return const LoginScreen();
      },
    );
  }
}
