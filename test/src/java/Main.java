import java.util.Scanner;

public class Main {
    public static void main(String[] args) {
        Scanner console = new Scanner(System.in);
        System.err.print("Enter your name: ");
        System.out.println("Hello, " + console.next() + "!");
    }
}