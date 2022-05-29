import java.util.Scanner;

public class Solution {
    private static int[][] getMatrix(Scanner in) {
        var rows = in.nextInt();
        var cols = in.nextInt();

        var res = new int[rows][cols];

        for (int i = 0; i < rows; i++) {
            for (int j = 0; j < cols; j++) {
                res[i][j] = in.nextInt();
            }
        }

        return res;
    }

    public static void main(String[] args) {
        var console = new Scanner(System.in);
        var first = getMatrix(console);
        var second = getMatrix(console);
        if (first[0].length != second.length) {
            System.out.println("Incompatible matrices!");
        } else {
            var res = new int[first.length][second[0].length];
            for (int i = 0; i < first.length; i++) {
                for (int j = 0; j < second[0].length; j++) {
                    for (int k = 0; k < second.length; k++) {
                        res[i][j] += first[i][k] * second[k][j];
                    }
                }
            }

            for (int i = 0; i < res.length; i++) {
                for (int j = 0; j < res[i].length; j++) {
                    System.out.printf("%4s", res[i][j]);
                }
                System.out.println();
            }
        }
    }
}