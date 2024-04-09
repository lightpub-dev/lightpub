import 'package:flutter/material.dart';

class LoginView extends StatefulWidget {
  const LoginView({super.key});

  @override
  State<LoginView> createState() => _LoginViewState();
}

class _LoginViewState extends State<LoginView> {
  @override
  Widget build(BuildContext context) {
    return Center(
        child: Column(children: [
      const SizedBox(
          width: double.infinity,
          child: TextField(
            decoration: InputDecoration(hintText: "Username"),
            autocorrect: false,
          )),
      const SizedBox(
          width: double.infinity,
          child: TextField(
            decoration: InputDecoration(hintText: "Password"),
            autocorrect: false,
            obscureText: true,
          )),
      ElevatedButton(
          onPressed: () {
            debugPrint("Login button pressed");
            
          },
          child: const Text('Login'))
    ]));
  }
}
