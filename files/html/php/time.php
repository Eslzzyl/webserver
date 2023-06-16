<!DOCTYPE html>
<html>
<head>
    <title>PHP基本功能示例</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            text-align: center;
        }
        
        h1 {
            color: #337ab7;
        }
        
        .result {
            font-size: 18px;
            margin-top: 20px;
        }
    </style>
</head>
<body>
    <h1>PHP基本功能示例</h1>
    
    <div class="result">
        <?php
            // 获取当前日期和时间
            $currentDateTime = date('Y-m-d H:i:s');
            echo "<p>当前日期和时间：$currentDateTime</p>";
            
            // 计算两个数字的和
            $num1 = 10;
            $num2 = 5;
            $sum = $num1 + $num2;
            echo "<p>两个数字的和：$sum</p>";
        ?>
    </div>
</body>
</html>