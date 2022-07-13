# factorial(n) =  n!
func factorial(n) -> (result):
    if n == 1:
        return (n)
    end
    let (a) = factorial(n-1)
    return (n*a)
end

func main():
    # Make sure the factorial(10) == 3628800
    let (y) = factorial(10)
    y = 3628800
    return ()
end
